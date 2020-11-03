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
use crate::opcodes::snes::*;
use crate::opcodes::buffer as op_buffer;

pub fn dump_rom<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions) {
    println!("IO_RESET");
    io::reset(&device_handle);
    println!("SNES_INIT");
    io::snes_init(&device_handle);

    // TODO: Support manual mapper lorom and hirom selection
    dump_snes_header(&device_handle);

}

pub fn dump_snes_header<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let hirom_header = get_header(&device_handle, 0x0000);
}

// Command line options
#[derive(Debug)]
pub struct SnesHeader {
    pub mapping: String,
    pub rom_name: String,
    pub map_mode: u8,
    pub rom_type: u8,
    pub rom_size: u8,
    pub sram_size: u8,
    pub exp_ram_size: u8,
    pub destination_code: u8,
    pub version: u8,
    pub compliment_check: u16,
    pub checksum: u16,
}

pub fn get_header<T: UsbContext>(device_handle: &DeviceHandle<T>, map_adjust: u16) {
    let addr_maker_code = 0xFFB0 - map_adjust; //             -- 2 bytes
    let addr_game_code = 0xFFB2 - map_adjust; //              -- 4 bytes
    let addr_fixed_zero = 0xFFB6 - map_adjust; //             -- 7 bytes
    let addr_expansion_ram_size = 0xFFBD - map_adjust; //     -- 1 byte
    let addr_special_version_code = 0xFFBE - map_adjust; //   -- 1 byte

    // ROM Specification Addresses (32 bytes)
    let addr_rom_name = 0xFFC0 - map_adjust; //               -- 21 bytes
    let addr_map_mode = 0xFFD5 - map_adjust; //               -- 1 byte
    let addr_rom_type = 0xFFD6 - map_adjust; //               -- 1 byte
    let addr_rom_size = 0xFFD7 - map_adjust; //               -- 1 byte
    let addr_sram_size = 0xFFD8 - map_adjust; //              -- 1 byte
    let addr_destination_code = 0xFFD9 - map_adjust; //       -- 1 byte
    let addr_developer_code = 0xFFDA - map_adjust; //         -- 1 byte (This is actually manufacturer ID)
    let addr_version = 0xFFDB - map_adjust; //                -- 1 byte
    let addr_compliment_check = 0xFFDC - map_adjust; //       -- 2 bytes
    let addr_checksum = 0xFFDD - map_adjust; //               -- 2 bytes

    let map_mode = rom_rd(&device_handle, addr_map_mode);
    let rom_type = rom_rd(&device_handle, addr_rom_type);
    let rom_size = rom_rd(&device_handle, addr_rom_size);
    let sram_size = rom_rd(&device_handle, addr_sram_size);
    let exp_ram_size = rom_rd(&device_handle, addr_expansion_ram_size);
    let destination_code = rom_rd(&device_handle, addr_destination_code);
    let developer_code = rom_rd(&device_handle, addr_developer_code);
    let version = rom_rd(&device_handle, addr_version);


    // mapping = mapping,
    // rom_name = string_from_bytes(addr_rom_name, 21),
    // map_mode = dict.snes("SNES_ROM_RD", addr_map_mode),
    // rom_type = dict.snes("SNES_ROM_RD", addr_rom_type),
    // rom_size = dict.snes("SNES_ROM_RD", addr_rom_size),
    // sram_size = dict.snes("SNES_ROM_RD", addr_sram_size),
    // exp_ram_size = dict.snes("SNES_ROM_RD", addr_expansion_ram_size),
    // destination_code = dict.snes("SNES_ROM_RD", addr_destination_code),
    // developer_code = dict.snes("SNES_ROM_RD", addr_developer_code),
    // version = dict.snes("SNES_ROM_RD", addr_version),
    // compliment_check = word_from_two_bytes(addr_compliment_check),
    // checksum = word_from_two_bytes(addr_checksum)


}


// Device functions

pub fn rom_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> u8 {
    let request = 4; // 4 is for SNES
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, request, SNES_ROM_RD, operand, 0);
    return buf[2];
}
