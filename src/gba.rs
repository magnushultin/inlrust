use rusb::{DeviceHandle, UsbContext};

use std::io::prelude::*;
use std::fs::File;
use std::io::BufWriter;

use crate::io;
use crate::util;

use crate::util::{CommandLineOptions, dump_to_array};
use crate::opcodes::gba::*;
use crate::opcodes::buffer as op_buffer;

// TODO: Check header checksum
pub fn dump_gba<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions) {
    io::reset(&device_handle);
    io::gba_init(&device_handle);
    io::gb_power_3v(&device_handle);

    let header = get_header(&device_handle);
    print_header(&header);

    if cmd_options.filename != "" {
        println!("Dumping ROM...");
        dump_rom(&device_handle, &cmd_options);
    }

    io::reset(&device_handle);
}

fn dump_rom<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions) {
    let file = File::create(&cmd_options.filename).unwrap();
    let mut f = BufWriter::new(file);


    let kb_per_read = 128; // 16 MBC or 32 rom only.
    let rom_size = 32 * 1024;
    let num_reads = rom_size / kb_per_read;
    let mut read_count = 0;
    // TODO Change to regular array?
    let mut dump_array = vec![0; (kb_per_read as usize) * 1024];

    while read_count < num_reads {
        if read_count % 8 == 0 {
            println!("dumping ROM bank: {} of {}", read_count, num_reads-1);
        }
        latch_addr(device_handle, 0x0000, read_count);
        
        dump_to_array(&device_handle, &mut dump_array, kb_per_read, 0x00, op_buffer::GBA_ROM_PAGE);
        release_bus(device_handle);
        
        match read_count {
            32  => if check_empty(&dump_array) { break; },
            64  => if check_empty(&dump_array) { break; },
            128 => if check_empty(&dump_array) { break; },
            _ => {},
        }

        f.write_all(&dump_array).unwrap();
        read_count +=  1;   
    }
}

// check if the whole array is 0xFF
fn check_empty(dump_array: &[u8]) -> bool {
    for i in dump_array {
        if *i != 0xff {
            return false;
        }
    }
    return true;
}

#[derive(Debug)]
pub struct GbaHeader {
    pub rom_name: String,     // 0x0A0 - 0x0AB Title
    pub game_code: String,   // 0x0AC - 0x0AF
    pub version: u8,  // 0x0BC
    pub header_checksum: u8,  // 0x0BD
}

fn get_header<T: UsbContext>(device_handle: &DeviceHandle<T>) -> GbaHeader {

    let mut header = GbaHeader{
        rom_name: "".to_string(),
        game_code: "".to_string(),
        version: 0,
        header_checksum: 0
    };

    // latch addr first        A0-15  A16-23
    latch_addr(&device_handle, 0x00A0 >> 1, 0x0000);
    header.rom_name = get_string_from_header(&device_handle, 12);
    release_bus(&device_handle);

    // latch addr first        A0-15  A16-23
    latch_addr(&device_handle, 0x00AC >> 1, 0x0000);
    header.game_code = get_string_from_header(&device_handle, 4);
    release_bus(&device_handle);

    latch_addr(&device_handle, 0x00BC >> 1, 0x0000);
    let tmp = rom_rd(device_handle); 
    header.version = (tmp & 0xFF) as u8;
    header.header_checksum = ((tmp >> 8) & 0xFF) as u8;
    release_bus(&device_handle);

    return header;
}

fn get_string_from_header<T: UsbContext>(device_handle: &DeviceHandle<T>, size: usize) -> String {
    let mut tmp_array = vec![0; size];
    get_byte_array(device_handle, &mut tmp_array);
    let rom_name = String::from_utf8_lossy(&tmp_array).to_string();
    return rom_name
}

fn get_byte_array<T: UsbContext>(device_handle: &DeviceHandle<T>, tmp_array: &mut [u8]) {
    for i in (0..tmp_array.len()).step_by(2) {
        let two = rom_rd(device_handle); 
        tmp_array[i] = (two & 0xFF) as u8;
        tmp_array[i + 1] = ((two >> 8) & 0xFF) as u8;
    }
}

fn print_header(header: &GbaHeader) {
    println!("------------ HEADER ------------");
    println!("Name: {}", header.rom_name);
    println!("Game code: {}", header.game_code);
    println!("Version: {}", header.version);
    println!("Header checksum: 0x{:X}", header.header_checksum);
    println!("--------------------------------");
}

// Device functions
fn rom_rd<T: UsbContext>(device_handle: &DeviceHandle<T>) -> u16 {
    let request = 13; // 13 is for gba
    let mut buf: [u8; 4] = [0; 4];
    util::read_device(device_handle, &mut buf, request, GBA_RD, 0, 0);
    let upper = (buf[3] as u16) << 8;
    let lower = buf[2] as u16; 
    return upper | lower;
}

pub fn latch_addr<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 13; // 13 is for GBA
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, GBA_LATCH_ADDR, operand, misc);
}

pub fn release_bus<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let request = 13; // 13 is for GBA
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, GBA_RELEASE_BUS, 0, 0);
}