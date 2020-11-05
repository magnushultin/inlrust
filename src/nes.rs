use rusb::{DeviceHandle, UsbContext};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

use crate::io;
use crate::pinport;
use crate::util;
use crate::util::CommandLineOptions;
use crate::opcodes::nes::*;
use crate::nes_mappers::{nrom, mmc1, unrom, cnrom, mmc3};

pub fn dump_nes<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions) {
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
        let kb_per_bank = 16;
        // Size is one byte smaller because table doesn't need fixed bank.
        let banktable_size = cmd_options.prg_size / kb_per_bank - 1;
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
