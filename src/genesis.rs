use rusb::{DeviceHandle, UsbContext};
use std::str;
use std::fs::File;
use std::io::BufWriter;
use std::convert::TryInto;

use crate::io;
use crate::util;

use crate::util::{CommandLineOptions, dump, dump_to_array};
use crate::opcodes::genesis::*;
use crate::opcodes::buffer as op_buffer;

// TODO: Check header checksum and global checksum

pub fn dump_genesis<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions) {
    io::reset(&device_handle);
    io::genesis_init(&device_handle);

    let header = get_header(&device_handle);
    print_header(&header);
    /*
    if cmd_options.savefile != "" {
        println!("Dumping save RAM...");
        dump_ram(&device_handle, &cmd_options, &header);
    }

    if cmd_options.filename != "" {
        println!("Dumping ROM...");
        dump_rom(&device_handle, &cmd_options, &header);
    }
    */
    io::reset(&device_handle);
}

#[derive(Debug)]
pub struct GenesisHeader {
    pub rom_name_domestic: String,
    pub rom_name_overseas: String,
    pub system_type: String,
    pub copyright: String,
    pub serial_number: String,
    pub checksum: u16,
    pub devices_supported: [u8; 16],
    pub rom_size: u32,
    pub ram_size: u32,
    pub extra_memory: bool,
    pub extra_memory_type: u8,
    pub extra_memory_size: u32,
    pub region_support: [u8; 3]
}

fn get_header<T: UsbContext>(device_handle: &DeviceHandle<T>) -> GenesisHeader {

    set_bank(device_handle, 0);
    
    let mut genesis_header = GenesisHeader{
        rom_name_domestic: "".to_string(),
        rom_name_overseas: "".to_string(),
        system_type: "".to_string(),
        copyright: "".to_string(),
        serial_number: "".to_string(),
        checksum: 0,
        devices_supported: [0; 16],
        rom_size: 0,
        ram_size: 0,
        extra_memory: false,
        extra_memory_type: 0,
        extra_memory_size: 0,
        region_support: [0; 3]
    };

    genesis_header.system_type = get_string_from_header(device_handle, 16, 0x100);
    genesis_header.copyright = get_string_from_header(device_handle, 16, 0x110);
    genesis_header.rom_name_domestic = get_string_from_header(device_handle, 48, 0x120);
    genesis_header.rom_name_overseas = get_string_from_header(device_handle, 48, 0x150);
    genesis_header.serial_number = get_string_from_header(device_handle, 14, 0x180);
    genesis_header.checksum = rom_rd(device_handle, (0x18E) >> 1);
    let mut tmp_array: [u8; 16] = [0; 16];
    get_byte_array(device_handle, &mut tmp_array, 0x190);
    genesis_header.devices_supported = tmp_array;

    let lower_addr = get_u32(device_handle, 0x1A0);
    let upper_addr = get_u32(device_handle, 0x1A4);
    genesis_header.rom_size = (upper_addr - lower_addr + 1) / 1024;

    // Always 64KiB. Does not indicate if there is save ram.
    let lower_addr = get_u32(device_handle, 0x1A8);
    let upper_addr = get_u32(device_handle, 0x1AC);
    genesis_header.ram_size = (upper_addr - lower_addr + 1) / 1024;

    let mut tmp_array: [u8; 12] = [0; 12];
    get_byte_array(device_handle, &mut tmp_array, 0x1B0);

    if tmp_array[0] as char == 'R' && tmp_array[1] as char == 'A' {
        genesis_header.extra_memory = true;
        genesis_header.extra_memory_type = tmp_array[2];

        let start = (tmp_array[4] as u32) << 24 | (tmp_array[5] as u32) << 16 | (tmp_array[6] as u32) << 8 | tmp_array[7] as u32;
        let end = (tmp_array[8] as u32) << 24 | (tmp_array[9] as u32) << 16 | (tmp_array[10] as u32) << 8 | tmp_array[11] as u32;
        genesis_header.extra_memory_size = (end - start + 1) / 1024;
    }
    
    let mut tmp_array: [u8; 4] = [0; 4];
    get_byte_array(device_handle, &mut tmp_array, 0x1F0);
    genesis_header.region_support.copy_from_slice(&tmp_array[..3]);

    return genesis_header;
}

fn get_u32<T: UsbContext>(device_handle: &DeviceHandle<T>, addr: u16) -> u32 {
    let hi = rom_rd(device_handle, (addr) >> 1);
    let lo = rom_rd(device_handle, (addr + 2) >> 1);
    let total: u32 = ((hi as u32) << 8) | lo as u32;
    return total;
}

fn get_string_from_header<T: UsbContext>(device_handle: &DeviceHandle<T>, size: usize, addr: u16) -> String {
    let mut tmp_array = vec![0; size];
    get_byte_array(device_handle, &mut tmp_array, addr);
    let rom_name = String::from_utf8_lossy(&tmp_array).to_string();
    return rom_name
}

fn get_byte_array<T: UsbContext>(device_handle: &DeviceHandle<T>, mut tmp_array: &mut [u8], addr: u16) {
    for i in (0..tmp_array.len()).step_by(2) {
        let two = rom_rd(device_handle, (addr + i as u16) >> 1); 
        tmp_array[i] = (two & 0xFF) as u8;
        tmp_array[i + 1] = ((two >> 8) & 0xFF) as u8;
    }
}

fn print_header(header: &GenesisHeader) {
    println!("------------ HEADER ------------");
    println!("Name (Domestic): {}", header.rom_name_domestic);
    println!("Name (Overseas): {}", header.rom_name_overseas);
    println!("ROM size: {} KiB", header.rom_size); // 32 KiB × (1 << <value>)
    println!("RAM size: {} KiB", header.ram_size);
    println!("System Type: {}", header.system_type);
    println!("Copyright: {}", header.copyright);
    println!("Serial number: {}", header.serial_number);
    println!("Extra memory support: {}", header.extra_memory);
    println!("Type of extra memory: {}", match_extra_memory_type(header.extra_memory_type).unwrap_or("Unknown"));
    println!("Extra memory size: {} KiB", header.extra_memory_size);
    println!("Region support: {}", String::from_utf8_lossy(&header.region_support));
    println!("--------------------------------");
}

// Device functions
fn rom_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> u16 {
    let request = 14; // 14 is for Sega
    let mut buf: [u8; 4] = [0; 4];
    util::read_device(device_handle, &mut buf, request, GEN_ROM_RD, operand, 0);
    let upper = (buf[2] as u16) << 8;
    let lower = buf[3] as u16; 
    return upper | lower;
}

fn set_bank<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) {
    let request = 14; // 14 is for Sega
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, GEN_SET_BANK, operand, 0);
}

fn match_extra_memory_type(mem_type: u8) -> Option<&'static str> {
    match mem_type {
        0xA0 => Some("No save 16-bit"),
        0xB0 => Some("No save 8-bit (even addresses)"),
        0xB8 => Some("No save 8-bit (odd addresses)"),
        0xE0 => Some("Save 16-bit"),
        0xE8 => Some("EEPROM"),
        0xF0 => Some("Save 8-bit (even addresses)"),
        0xF8 => Some("Save 8-bit (odd addresses)"),
        _ => None
    }
}