use rusb::{DeviceHandle, UsbContext};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

use crate::io;
use crate::operation;
use crate::buffer;
use crate::nes::{discrete_exp0_prgrom_wr, detect_mapper_mirroring, cpu_rd, cpu_wr, dump, ppu_ram_sense, buffer_allocate};
use crate::opcodes::buffer as op_buffer;

pub fn test_unrom<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    println!("Testing UNROM");
    println!("Detect mapper mirroring");
    detect_mapper_mirroring(&device_handle);

    ppu_ram_sense(&device_handle, 0x1000);
    println!("EXP0 pull-up test: {}", io::exp0_pullup_test(&device_handle));

    //    read PRG-ROM manf ID
    // init mapper
    cpu_wr(&device_handle, 0x8000, 0x00);

    discrete_exp0_prgrom_wr(&device_handle, 0x5555, 0xAA);
    discrete_exp0_prgrom_wr(&device_handle, 0x2AAA, 0x55);
    discrete_exp0_prgrom_wr(&device_handle, 0x5555, 0x90);

    let rv = cpu_rd(&device_handle, 0x8000);
    println!("PRG-ROM manf ID: 0x{:x}", rv);

    let rv = cpu_rd(&device_handle, 0x8001);
    println!("PRG-ROM prod ID: 0x{:x}", rv);

    // Exit
    discrete_exp0_prgrom_wr(&device_handle, 0x8000, 0xF0);
}

pub fn find_banktable<T: UsbContext>(device_handle: &DeviceHandle<T>, banktable_size: u8) -> u16 {
    let search_base = 0x0C; // search in $C000-$F000, the fixed bank
    const KB_search_space: u16 = 16;
    let mut full_dump: [u8; 128 * (KB_search_space as usize * 1024 / 128)] = [0; 128 * (KB_search_space as usize * 1024 / 128)];


    let size_kb: u16 = KB_search_space;
    let map: u16 = search_base;
    let mem: u16 = op_buffer::NESCPU_4KB;
    let buff0 = 0;
    let buff1 = 1;
    println!("SET_OPERATION RESET");
    operation::set_operation(&device_handle, 0x01);
    println!("RAW_BUFFER_RESET");
    buffer::raw_buffer_reset(&device_handle);
    buffer_allocate(&device_handle, 2, 128);
    buffer::set_mem_n_part(&device_handle, (mem << 8) | 0xDD, buff0);
    buffer::set_mem_n_part(&device_handle, (mem << 8) | 0xDD, buff1);
    buffer::set_map_n_mapvar(&device_handle, (map << 8) | 0, buff0);
    buffer::set_map_n_mapvar(&device_handle, (map << 8) | 0, buff1);

    println!("SET_OPERATION STARTDUMP");
    operation::set_operation(&device_handle, 0xD2);

    let mut buf: [u8; 128] = [0; 128];
    let mut buff_status = 0;
    println!("sizeKB*1024/buff_size={}", size_kb * 1024 / 128);
    for i in 0..(size_kb * 1024 / 128) {
        for try_nbr in 0..20 {
            buff_status = buffer::get_cur_buff_status(&device_handle);
            // DUMPED = 0xD8
            if buff_status == 0xD8 {
                break;
            }
        }
        if buff_status != 0xD8 {
            println!("DID NOT GET BUFF STATUS");
            println!("STOPPING!");
            break;
        }

        buffer::buff_payload(&device_handle, &mut buf);
        full_dump[i as usize * 128..(i as usize + 1) * 128].copy_from_slice(&buf);
    }

    println!("SET_OPERATION RESET");
    operation::set_operation(&device_handle, 0x01);
    println!("RAW_BUFFER_RESET");
    buffer::raw_buffer_reset(&device_handle);

    let max_consec = banktable_size;
    let mut current_val: u8 = 0;
    let mut number_of_consecutive = 0;
    let mut potential_index = 0;

    for (i, byte) in full_dump.iter().enumerate() {
        if *byte == current_val {
            if number_of_consecutive == 0 {
                potential_index = i;
            }
            number_of_consecutive += 1;
            current_val += 1;
        } else {
            potential_index = 0;
            number_of_consecutive = 0;
            current_val = 0;
        }
        if current_val == max_consec {
            break;
        }
    }
    return 0xC000 + (potential_index as u16);
}

pub fn dump_prgrom_unrom<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u16,
    banktable_base: u16
) {
    let mut kb_per_read = 16;
    let num_reads = rom_size_kb / kb_per_read;
    let mut read_count = 0;
    let addr_base = 0x08;
    let fixed_bank_base = 0x0C;

    while read_count < num_reads - 1 {
        cpu_wr(&device_handle, banktable_base + read_count, read_count);

        dump(&device_handle, file, kb_per_read, addr_base, op_buffer::NESCPU_4KB);
        read_count += 1;
    }

    dump(&device_handle, file, kb_per_read, fixed_bank_base, op_buffer::NESCPU_4KB);
}
