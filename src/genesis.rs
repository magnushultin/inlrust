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
    pub extra_memory: [u8; 12], // SRAM support, or EEPROM
    pub modem_support: [u8; 12],
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
        extra_memory: [0; 12],
        modem_support: [0; 12],
        region_support: [0; 3]
    };

   
    // same thing with rom_rd
    // Name is 48 bytes, 2 bytes per read
    let mut tmp_array: [u8; 48] = [0; 48];
    for i in (0..48).step_by(2) {
        let two = rom_rd(device_handle, (0x120 + i as u16) >> 1); 
        tmp_array[i] = (two & 0xFF) as u8;
        tmp_array[i + 1] = ((two >> 8) & 0xFF) as u8;
    }
    let rom_name = String::from_utf8_lossy(&tmp_array).to_string();
    genesis_header.rom_name_domestic = rom_name;

    // TODO: get the rest of the header.
    return genesis_header;
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