use rusb::{DeviceHandle, UsbContext};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

use crate::io;
use crate::buffer;
use crate::operation;
use crate::pinport;
use crate::util;
use crate::util::CommandLineOptions;
use crate::opcodes::nes::*;
use crate::opcodes::buffer as op_buffer;
use crate::nes_mappers::{nrom, mmc1, unrom, cnrom, mmc3};

pub fn dump_rom<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions) {
    println!("IO_RESET");
    io::reset(&device_handle);
    // NES INIT
    println!("NES_INIT");
    io::nes_init(&device_handle);

    if cmd_options.mapper.to_lowercase() == "nrom" {
        nrom::test_nrom(&device_handle);
        // MIRROR
        //   detect_mapper_mirroring
        //   ciccom
        // READ
        let file = File::create(&cmd_options.filename).unwrap();
        let mut f = BufWriter::new(file);

        //   create_header
        let mirroring = detect_mapper_mirroring(&device_handle).unwrap();
        create_header(&mut f, cmd_options.prg_size, cmd_options.chr_size, 0, mirroring);
        nrom::dump_prgrom(&device_handle, &mut f, cmd_options.prg_size);
        nrom::dump_chrrom(&device_handle, &mut f, cmd_options.chr_size);

        f.flush().unwrap();
    } else if cmd_options.mapper.to_lowercase() == "mmc1" {
        mmc1::test_mmc1(&device_handle);

        mmc1::init_mapper_mmc1(&device_handle);
        let file = File::create(&cmd_options.filename).unwrap();
        let mut f = BufWriter::new(file);
        create_header(&mut f, cmd_options.prg_size, cmd_options.chr_size, 1, Mirroring::HORZ);
        mmc1::dump_prgrom_mmc1(&device_handle, &mut f, cmd_options.prg_size);
        mmc1::dump_chrrom_mmc1(&device_handle, &mut f, cmd_options.chr_size);

        f.flush().unwrap();
    } else if cmd_options.mapper.to_lowercase() == "unrom" {
        unrom::test_unrom(&device_handle);

        // find bank table to avoid bus conflicts
        let KB_per_bank = 16;
        // Size is one byte smaller because table doesn't need fixed bank.
        let banktable_size = cmd_options.prg_size / KB_per_bank - 1;
        let banktable_base = unrom::find_banktable(&device_handle, banktable_size as u8);
        println!("Found banktable addr = {}", banktable_base);

        let file = File::create(&cmd_options.filename).unwrap();
        let mut f = BufWriter::new(file);
        let mirroring = detect_mapper_mirroring(&device_handle).unwrap();
        create_header(&mut f, cmd_options.prg_size, cmd_options.chr_size, 2, mirroring);
        unrom::dump_prgrom_unrom(&device_handle, &mut f, cmd_options.prg_size, banktable_base);

        f.flush().unwrap();
    } else if cmd_options.mapper.to_lowercase() == "cnrom" {
        cnrom::test_cnrom(&device_handle);

        let file = File::create(&cmd_options.filename).unwrap();
        let mut f = BufWriter::new(file);

        let mirroring = detect_mapper_mirroring(&device_handle).unwrap();
        create_header(&mut f, cmd_options.prg_size, cmd_options.chr_size, 3, mirroring);
        cnrom::dump_prgrom(&device_handle, &mut f, cmd_options.prg_size);
        cnrom::dump_chrrom(&device_handle, &mut f, cmd_options.chr_size);

        f.flush().unwrap();
    } else if cmd_options.mapper.to_lowercase() == "mmc3" {
        mmc3::test_mmc3(&device_handle);

        let file = File::create(&cmd_options.filename).unwrap();
        let mut f = BufWriter::new(file);

        mmc3::init_mapper(&device_handle);
        let mirroring = detect_mapper_mirroring(&device_handle).unwrap();
        create_header(&mut f, cmd_options.prg_size, cmd_options.chr_size, 4, mirroring);
        mmc3::dump_prgrom(&device_handle, &mut f, cmd_options.prg_size);
        mmc3::dump_chrrom(&device_handle, &mut f, cmd_options.chr_size);

        f.flush().unwrap();
    } else {
        println!("Mapper {} is not supported!", cmd_options.mapper);
    }
}

pub fn dump<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    size_kb: u16,
    map: u16,
    mem: u16,
) {
    //local buff0 = 0
    let buff0 = 0;
    //local buff1 = 1
    let buff1 = 1;
    //local cur_buff_status = 0
    // let curr_buff_status = 0;
    //local data = nil --lua stores data in strings
    //
    //if debug then print("dumping cart") end
    println!("Dumping cart");

    //dict.operation("SET_OPERATION", op_buffer["RESET"] )
    // shared_dict_buffer #define RESET		0x01
    // TODO: Change buffer/operation operands to enum
    println!("SET_OPERATION RESET");
    operation::set_operation(&device_handle, 0x01);

    //--reset buffers first
    //dict.buffer("RAW_BUFFER_RESET")
    println!("RAW_BUFFER_RESET");
    buffer::raw_buffer_reset(&device_handle);
    //local data = nil --lua stores data in strings
    //
    //if debug then print("dumping cart") end
    println!("Dumping cart");

    //--need to allocate some buffers for dumping
    //--2x 128Byte buffers
    //local num_buffers = 2
    //local buff_size = 128
    //if debug then print("allocating buffers") end
    //assert(buffers.allocate( num_buffers, buff_size ), "fail to allocate buffers")
    buffer_allocate(&device_handle, 2, 128);
    // op_buffer[mem] (op_buffer["NESCPU_4KB"]) = 0x20
    // NROM = mapper 0
    // op_buffer[NOVAR] = 0
    // op_buffer["MASKROM"] = 0xDD

    // if debug then print("setting map n part") end
    // dict.buffer("SET_MEM_N_PART", (op_buffer[mem]<<8 | op_buffer["MASKROM"]), buff0 )
    // dict.buffer("SET_MEM_N_PART", (op_buffer[mem]<<8 | op_buffer["MASKROM"]), buff1 )
    buffer::set_mem_n_part(&device_handle, (mem << 8) | 0xDD, buff0);
    buffer::set_mem_n_part(&device_handle, (mem << 8) | 0xDD, buff1);

    // dict.buffer("SET_MAP_N_MAPVAR", (mapper<<8 | op_buffer["NOVAR"]), buff0 )
    // address base = 0x08  -- $8000
    buffer::set_map_n_mapvar(&device_handle, (map << 8) | 0, buff0);
    buffer::set_map_n_mapvar(&device_handle, (map << 8) | 0, buff1);

    // op_buffer[STARTDUMP] = 0xD2
    println!("SET_OPERATION STARTDUMP");
    operation::set_operation(&device_handle, 0xD2);

    // for i=1, (sizeKB*1024/buff_size)
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

        file.write_all(&buf).unwrap();
    }

    println!("DUMPING DONE!");

    println!("SET_OPERATION RESET");
    operation::set_operation(&device_handle, 0x01);
    println!("RAW_BUFFER_RESET");
    buffer::raw_buffer_reset(&device_handle);
}

//app/buffers.lua allocate()
pub fn buffer_allocate<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    num_buffers: u16,
    buff_size: u16,
) {
    let buff0basebank = 0;
    // shared_dict_buffer.h:  #define RAW_BANK_SIZE   32
    // local numbanks = buff_size/ (op_buffer["RAW_BANK_SIZE"])
    let numbanks = buff_size / 32;
    let buff1basebank = numbanks;

    let mut buff0id = 0;
    let mut buff1id = 0;
    let mut reload = 0;
    let mut buff0_firstpage = 0;
    let mut buff1_firstpage = 0;

    if (num_buffers == 2) && (buff_size == 128) {
        //buff0 dumps first half of page, buff1 dumps second half, repeat
        //MSB tells buffer value of A7 when operating
        buff0id = 0x00;
        buff1id = 0x80;
        //set reload (value added to page_num after each load/dump to sum of buffers
        // 2 * 128 = 256 -> reload = 1
        reload = 0x01;
        //set first page
        buff0_firstpage = 0x0000;
        buff1_firstpage = 0x0000;
    } else if (num_buffers == 2) && (buff_size == 256) {
        //buff0 dumps even pages, buff1 dumps odd pages
        //buffer id not used for addressing both id zero for now..
        buff0id = 0x00;
        buff1id = 0x00;
        //set reload (value added to page_num after each load/dump to sum of buffers
        // 2 * 256 = 512 -> reload = 2
        reload = 0x02;
        //set first page of each buffer
        buff0_firstpage = 0x0000;
        buff1_firstpage = 0x0001;
    } else {
        println!("ERROR! Not setup to handle this buffer config");
    }
    println!("Buffer allocate buffer0");
    buffer::allocate_buffer0(&device_handle, (buff0id << 8) | buff0basebank, numbanks);
    println!("Buffer allocate buffer1");
    buffer::allocate_buffer1(&device_handle, (buff1id << 8) | buff1basebank, numbanks);
    println!("Buffer set reload pagenum0");
    buffer::set_reload_pagenum0(&device_handle, buff0_firstpage, reload);
    println!("Buffer set reload pagenum1");
    buffer::set_reload_pagenum1(&device_handle, buff1_firstpage, reload);
}

// General NES functions

fn create_header<W: Write>(
    file: &mut BufWriter<W>,
    prg_size: u16,
    chr_size: u16,
    mapper: u8,
    mirroring: Mirroring,
) {
    file.write(b"NES").unwrap();

    file.write_all(&[0x1A]).unwrap();
    // byte 4
    file.write_all(&[(prg_size / 16) as u8]).unwrap();
    // byte 5
    file.write_all(&[(chr_size / 8) as u8]).unwrap();

    // byte 6
    let mut temp = mapper & 0x0F;
    temp = temp << 4;

    if mirroring == Mirroring::VERT {
        temp = temp | 0x01;
    }
    file.write_all(&[temp]).unwrap();

    // byte 7
    let temp = mapper & 0xF0;
    file.write_all(&[temp]).unwrap();

    // byte 8-15
    file.write_all(&[0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
}

#[derive(Eq, PartialEq)]
pub enum Mirroring {
    VERT,
    HORZ,
    SCNA,
    SCNB,
}

pub fn detect_mapper_mirroring<T: UsbContext>(device_handle: &DeviceHandle<T>) -> Result<Mirroring, String> {
    // TODO: call mmc3 detection function
    // TODO: call mmc1 detection function
    // TODO: fme7 and other ASIC mappers

    // PINPORT ADDR_SET, 0x0800
    //   1       17      0x0800
    println!("PINPORT_ADDR_SET 0x0800");
    pinport::addr_set(&device_handle, 0x0800);
    // readH = PINPORT CTL_RD, CIA10         RL=4 (err_code, data_len, LSB, MSB)
    //           1       6       11
    let read_h = pinport::ctl_rd(&device_handle, 11).unwrap();
    println!("Read h: {}", read_h);

    // PINPORT ADDR_SET, 0x0400
    println!("PINPORT_ADDR_SET 0x0400");
    pinport::addr_set(&device_handle, 0x0400);
    // readH = PINPORT CTL_RD, CIA10         RL=4 (err_code, data_len, LSB, MSB)
    //           1       6       11
    let read_v = pinport::ctl_rd(&device_handle, 11).unwrap();

    if read_v == 0 && read_h == 0 {
        println!("1SCNA - 1screen A mirroring");
        return Ok(Mirroring::SCNA);
    } else if read_v != 0 && read_h == 0 {
        println!("VERT - Vertical mirroring");
        return Ok(Mirroring::VERT);
    } else if read_v == 0 && read_h != 0 {
        println!("HORZ - Horizontal mirroring");
        return Ok(Mirroring::HORZ);
    } else {
        println!("1SCNB - 1screen B mirroring");
        return Ok(Mirroring::SCNB);
    };
}

pub fn ppu_ram_sense<T: UsbContext>(device_handle: &DeviceHandle<T>, addr: u16) -> bool {
    ppu_wr(&device_handle, addr, 0xAA);
    if ppu_rd(&device_handle, addr) != 0xAA {
        println!("Could not write 0xAA to PPU {:X}", addr);
        return false;
    }

    ppu_wr(&device_handle, addr, 0x55);
    if ppu_rd(&device_handle, addr) != 0x55 {
        println!("Could not write 0x55 to PPU {:X}", addr);
        return false;
    }

    println!("Detected RAM @ PPU {:X}", addr);
    return true;
}

// Device functions

pub fn discrete_exp0_prgrom_wr<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    operand: u16,
    misc: u16,
) {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(
        device_handle,
        &mut buf,
        request,
        DISCRETE_EXP0_PRGROM_WR,
        operand,
        misc,
    );
}

pub fn cpu_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> u8 {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, request, NES_CPU_RD, operand, 0);
    return buf[2];
}

pub fn cpu_wr<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, NES_CPU_WR, operand, misc);
}

pub fn mmc1_wr<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, NES_MMC1_WR, operand, misc);
}

pub fn ppu_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> u8 {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, request, NES_PPU_RD, operand, 0);
    return buf[2];
}

pub fn ppu_wr<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, NES_PPU_WR, operand, misc);
}
