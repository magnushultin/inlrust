use rusb::{DeviceHandle, UsbContext};
use std::io::prelude::*;
use std::io::BufWriter;

use crate::io;
use crate::util::dump;
use crate::nes::{discrete_exp0_prgrom_wr, detect_mapper_mirroring, ppu_wr, ppu_rd, cpu_rd};
use crate::opcodes::buffer as op_buffer;

pub fn test_nrom<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    println!("Testing NROM");
    println!("Detect mapper mirroring");
    detect_mapper_mirroring(&device_handle).unwrap();
    //    IO EXP0_PULLUP_TEST
    io::exp0_pullup_test(&device_handle);
    //    read PRG-ROM manf ID
    discrete_exp0_prgrom_wr(&device_handle, 0x5555, 0xAA);
    discrete_exp0_prgrom_wr(&device_handle, 0x2AAA, 0x55);
    discrete_exp0_prgrom_wr(&device_handle, 0x5555, 0x90);

    let rv = cpu_rd(&device_handle, 0x8000);
    println!("PRG-ROM manf ID: 0x{:x}", rv);

    let rv = cpu_rd(&device_handle, 0x8001);
    println!("PRG-ROM prod ID: 0x{:x}", rv);

    // Exit
    discrete_exp0_prgrom_wr(&device_handle, 0x8000, 0xF0);

    //    read CHR-ROM manf ID
    ppu_wr(&device_handle, 0x1555, 0xAA);
    ppu_wr(&device_handle, 0x0AAA, 0x55);
    ppu_wr(&device_handle, 0x1555, 0x90);

    let rv = ppu_rd(&device_handle, 0x0000);
    println!("CHR-ROM manf ID: 0x{:x}", rv);

    let rv = ppu_rd(&device_handle, 0x0001);
    println!("CHR-ROM prod ID: 0x{:x}", rv);
    // EXIT
    ppu_wr(&device_handle, 0x0000, 0xF0);
}

pub fn dump_prgrom<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u16,
) {
    let mut kb_per_read = 32;

    // Handle 16KB nroms.
    if rom_size_kb < kb_per_read {
        kb_per_read = rom_size_kb;
    }
    let num_reads = rom_size_kb / kb_per_read;
    let mut read_count = 0;
    let addr_base = 0x08;

    // NESCPU_4KB = 0x20
    while read_count < num_reads {
        dump(&device_handle, file, kb_per_read, addr_base, op_buffer::NESCPU_4KB);
        read_count += 1;
    }
}

pub fn dump_chrrom<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u16,
) {
    let kb_per_read = 8;
    let num_reads = rom_size_kb / kb_per_read;
    let mut read_count = 0;
    let addr_base = 0x00;

    // NESPPU_1KB = 0x21
    while read_count < num_reads {
        dump(&device_handle, file, kb_per_read, addr_base, op_buffer::NESPPU_1KB);
        read_count += 1;
    }
}
