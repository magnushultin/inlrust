use rusb::{DeviceHandle, UsbContext};
use std::io::prelude::*;
use std::io::BufWriter;

use crate::io;
use crate::util::dump;
use crate::nes::{detect_mapper_mirroring, ppu_wr, ppu_rd, cpu_rd, cpu_wr, mmc1_wr, Mirroring, ppu_ram_sense};
use crate::opcodes::buffer as op_buffer;

pub fn test_mmc1<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    println!("Testing MMC1");
    //mirror_test
    init_mapper_mmc1(&device_handle);

    // Screen A
    mmc1_wr(&device_handle, 0x8000, 0x00);
    if detect_mapper_mirroring(&device_handle).unwrap() != Mirroring::SCNA {
        println!("MMC1 mirror test fail (1 screen A)");
    }

    mmc1_wr(&device_handle, 0x8000, 0x01);
    if detect_mapper_mirroring(&device_handle).unwrap() != Mirroring::SCNB {
        println!("MMC1 mirror test fail (1 screen B)");
    }

    mmc1_wr(&device_handle, 0x8000, 0x02);
    if detect_mapper_mirroring(&device_handle).unwrap() != Mirroring::VERT {
        println!("MMC1 mirror test fail (Vertical)");
    }

    mmc1_wr(&device_handle, 0x8000, 0x03);
    if detect_mapper_mirroring(&device_handle).unwrap() != Mirroring::HORZ {
        println!("MMC1 mirror test fail (Horizontal)");
    }

    ppu_ram_sense(&device_handle, 0x1000);
    println!("EXP0 pull-up test: {}", io::exp0_pullup_test(&device_handle));

    // prgrom manf id
    init_mapper_mmc1(&device_handle);
    cpu_wr(&device_handle, 0xD555, 0xAA);
    cpu_wr(&device_handle, 0xAAAA, 0x55);
    cpu_wr(&device_handle, 0xD555, 0x90);

    let rv = cpu_rd(&device_handle, 0x8000);
    println!("PRG-ROM manf ID: 0x{:x}", rv);

    let rv = cpu_rd(&device_handle, 0x8001);
    println!("PRG-ROM prod ID: 0x{:x}", rv);

    // Exit
    cpu_wr(&device_handle, 0x8000, 0xF0);

    //    read CHR-ROM manf ID
    init_mapper_mmc1(&device_handle);
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

pub fn init_mapper_mmc1<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    cpu_rd(&device_handle, 0x8000);
    cpu_wr(&device_handle, 0x8000, 0x80);
    mmc1_wr(&device_handle, 0x8000, 0x10);
    mmc1_wr(&device_handle, 0xE000, 0x10);
    mmc1_wr(&device_handle, 0xA000, 0x12);
    mmc1_wr(&device_handle, 0xC000, 0x15);
}

pub fn dump_prgrom_mmc1<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u16,
) {
    let kb_per_read = 32;
    let num_reads = rom_size_kb / kb_per_read;
    let mut read_count = 0;
    let addr_base = 0x08;

    while read_count < num_reads {
        mmc1_wr(&device_handle, 0xE000, read_count<<1);
        dump(&device_handle, file, kb_per_read, addr_base, op_buffer::NESCPU_4KB);
        read_count += 1;
    }
}

pub fn dump_chrrom_mmc1<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u16,
) {
    let kb_per_read = 8;
    let num_reads = rom_size_kb / kb_per_read;
    let mut read_count = 0;
    let addr_base = 0x00;

    while read_count < num_reads {
        mmc1_wr(&device_handle, 0xA000, read_count*2);
        mmc1_wr(&device_handle, 0xC000, read_count*2+1);
        // NESPPU_1KB = 0x21
        dump(&device_handle, file, kb_per_read, addr_base, op_buffer::NESPPU_1KB);
        read_count += 1;
    }
}
