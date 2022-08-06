use rusb::{DeviceHandle, UsbContext};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

use crate::io;
use crate::util;
use crate::util::{CommandLineOptions, dump};
use crate::opcodes::snes::*;
use crate::opcodes::buffer as op_buffer;

pub fn dump_snes<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions) {
    io::reset(&device_handle);
    io::snes_init(&device_handle);

    let header = dump_snes_header(&device_handle).unwrap();
    print_header(&header);

    let snes_mapping = match_map_mode(header.map_mode).unwrap();
    println!("Autodetected {} mapping", snes_mapping);

    let rombank;
    let rambank;
    if snes_mapping == "HiROM" {
        rombank = 0xC0;
        rambank = 0x30;
    } else if snes_mapping == "LoROM" {
        rombank = 0x00;
        rambank = 0x70;
    } else {
        println!("Unsupported mapper: {}", snes_mapping);
        return;
    }

    // detect sram size
    let mut ram_size = 0;
    if header.sram_size < 7 {
        ram_size = 2_u16.pow(header.sram_size.into());
    }
    println!("ram_size: {} kilobytes", ram_size);

    let mut exp_ram_size = 0;
    if header.exp_ram_size < 7 {
        exp_ram_size = 2_u16.pow(header.exp_ram_size.into());
    }
    println!("exp_ram_size: {} kilobytes", exp_ram_size);
    if ram_size == 0 && exp_ram_size  > 0 {
        println!("ram size will be the exp ram size");
        ram_size = exp_ram_size;
    }

    // detect rom size
    let test_string: String = format!("Unknown: 0x{:X}", header.rom_size);
    let rom_size = match_rom_size_kb(header.rom_size).expect(&test_string);
    println!("rom_size: {} kilobytes", rom_size);

    println!("{:?}", cmd_options);
    if cmd_options.savefile != "" {
        println!("Dumping SAVE RAM...");

        println!("rambank {}", rambank);
        println!("ram_size {}", ram_size);
        println!("snes_mapping {}", snes_mapping);
        dump_ram(&device_handle, &cmd_options, rambank, ram_size, snes_mapping);
    }

    if cmd_options.filename != "" {
        println!("Dumping SNES ROM...");

        dump_rom(&device_handle, &cmd_options, rombank, rom_size, snes_mapping);
    }



}

fn dump_rom<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions,
    start_bank: u16, rom_size: u16, snes_mapping: &str) {

	let kb_per_bank;
	let addr_base;

        if snes_mapping == "HiROM" {
            kb_per_bank = 64; // 64KB per bank
            addr_base = 0x00;
        } else if snes_mapping == "LoROM" {
            kb_per_bank = 32;
            addr_base = 0x80;
        } else {
            println!("Unsupported mapping: {}", snes_mapping);
            return;
        }

        let file = File::create(&cmd_options.filename).unwrap();
        let mut f = BufWriter::new(file);

	let num_reads = rom_size / kb_per_bank;
	let mut read_count = 0;

	while read_count < num_reads {

            if read_count % 8 == 0 {
                println!("dumping ROM bank: {} of {}", read_count, num_reads-1);
            }
            // select desired bank
            set_bank(&device_handle, start_bank + read_count);

            dump(&device_handle, &mut f, kb_per_bank, addr_base, op_buffer::SNESROM_PAGE);

            read_count +=  1
        }

        f.flush().unwrap();
}

fn dump_ram<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions,
    start_bank: u16, ram_size: u16, snes_mapping: &str) {

	let mut kb_per_bank;
	let addr_base;

        if snes_mapping == "HiROM" {
            kb_per_bank = 8; // 8KB per bank
            addr_base = 0x60;
        } else if snes_mapping == "LoROM" {
            kb_per_bank = 32;
            addr_base = 0x00;
        } else {
            println!("Unsupported mapping: {}", snes_mapping);
            return;
        }

        let file = File::create(&cmd_options.savefile).unwrap();
        let mut f = BufWriter::new(file);

        let num_banks;
        if ram_size < kb_per_bank {
            num_banks = 1;
            kb_per_bank = ram_size;
        } else {
            num_banks = ram_size / kb_per_bank;
        }

	let mut read_count = 0;
	while read_count < num_banks {

            println!("dump RAM part {} of {}", read_count, num_banks);

            // select desired bank
            println!("set bank start_bank: {} read_count: {}", start_bank, read_count);
            set_bank(&device_handle, start_bank + read_count);

            if snes_mapping == "LoROM" {
                println!("LoROM, kb_per_bank: {} , addr_base: {}", kb_per_bank, addr_base);
                dump(&device_handle, &mut f, kb_per_bank, addr_base, op_buffer::SNESROM_PAGE);
            } else {
                dump(&device_handle, &mut f, kb_per_bank, addr_base, op_buffer::SNESSYS_PAGE);
            }
            read_count +=  1
        }

        f.flush().unwrap();
}

fn match_rom_size_kb(rom_size: u8) -> Option<u16> {
    match rom_size {
        0x08 => Some(2 * 128),
        0x09 => Some(4 * 128),
        0x0A => Some(8 * 128),
        0x0B => Some(16 * 128),
        0x0C => Some(32 * 128),
        0x0D => Some(64 * 128),
        _ => None
    }
}

fn print_header(header: &SnesHeader) {
    println!("------------ HEADER ------------");
    println!("Name: {}", header.rom_name);
    println!("map_mode: {}", match_map_mode(header.map_mode).unwrap_or("Unknown"));
    println!("rom_speed: {}", if (header.map_mode & 0x10) == 0x10 { "Fast 120ns" } else { "Slow 200ns" });
    println!("rom_type: {}", match_hardware_type(header.rom_type).unwrap_or("Unknown"));
    println!("upper bound rom_size: {}", match_rom_upper_bound(header.rom_size).unwrap_or("Unknown"));
    println!("sram_size: {}", match_ram_size(header.sram_size).unwrap_or("Unknown"));
    let test_string: String = format!("Unknown: 0x{:X}", header.exp_ram_size);
    println!("exp_ram_size: {}", match_ram_size(header.exp_ram_size).unwrap_or(&test_string));
    println!("destination: {}", match_destination(header.destination_code).unwrap_or("Unknown"));
    println!("developer: {}", match_developer(header.developer_code).unwrap_or("Unknown"));
    println!("version: 0x{:X}", header.version);
    println!("compliment check: {:X}", header.compliment_check);
    println!("checksum: {:X}", header.checksum);
    println!("--------------------------------");
}


fn is_valid_header(header: &SnesHeader) -> bool {
    if match_hardware_type(header.rom_type).is_none() {
        return false;
    }
    if match_destination(header.destination_code).is_none() {
        return false;
    }
    if match_rom_upper_bound(header.rom_size).is_none() {
        return false;
    }
    if match_ram_size(header.sram_size).is_none() {
        return false;
    }
    return true;
}

fn dump_snes_header<T: UsbContext>(device_handle: &DeviceHandle<T>) -> Result<SnesHeader, &'static str> {
    let hirom_header = get_header(&device_handle, 0x0000);
    let lorom_header = get_header(&device_handle, 0x8000);
    let exhirom_header = get_header(&device_handle, -0x400000);
    if is_valid_header(&exhirom_header) {
        println!("Valid header found at exHiROM address.");
        return Ok(exhirom_header);
    } else if is_valid_header(&hirom_header) {
        println!("Valid header found at HiROM address.");
        return Ok(hirom_header);
    } else if is_valid_header(&lorom_header) {
        println!("Valid header found at LoROM address.");
        return Ok(lorom_header);
    }

    return Err("Could not parse internal ROM header.");
}

// Command line options
#[derive(Debug)]
pub struct SnesHeader {
    pub rom_name: String,
    pub map_mode: u8,
    pub rom_type: u8,
    pub rom_size: u8,
    pub sram_size: u8,
    pub exp_ram_size: u8,
    pub destination_code: u8,
    pub developer_code: u8,
    pub version: u8,
    pub compliment_check: u16,
    pub checksum: u16,
}
use std::str;

fn get_header<T: UsbContext>(device_handle: &DeviceHandle<T>, map_adjust: i32) -> SnesHeader {
    let addr_expansion_ram_size = (0xFFBD - map_adjust) as u16; //     -- 1 byte

    // ROM Specification Addresses (32 bytes)
    let addr_rom_name         = (0xFFC0 - map_adjust) as u16; //               -- 21 bytes
    let addr_map_mode         = (0xFFD5 - map_adjust) as u16; //               -- 1 byte
    let addr_rom_type         = (0xFFD6 - map_adjust) as u16; //               -- 1 byte
    let addr_rom_size         = (0xFFD7 - map_adjust) as u16; //               -- 1 byte
    let addr_sram_size        = (0xFFD8 - map_adjust) as u16; //              -- 1 byte
    let addr_destination_code = (0xFFD9 - map_adjust) as u16; //       -- 1 byte
    let addr_developer_code   = (0xFFDA - map_adjust) as u16; //         -- 1 byte (This is actually manufacturer ID)
    let addr_version          = (0xFFDB - map_adjust) as u16; //                -- 1 byte
    let addr_compliment_check = (0xFFDC - map_adjust) as u16; //       -- 2 bytes
    let addr_checksum         = (0xFFDD - map_adjust) as u16; //               -- 2 bytes

    let map_mode = rom_rd(&device_handle, addr_map_mode);
    let rom_type = rom_rd(&device_handle, addr_rom_type);
    let rom_size = rom_rd(&device_handle, addr_rom_size);
    let sram_size = rom_rd(&device_handle, addr_sram_size);
    let exp_ram_size = rom_rd(&device_handle, addr_expansion_ram_size);
    let destination_code = rom_rd(&device_handle, addr_destination_code);
    let developer_code = rom_rd(&device_handle, addr_developer_code);
    let version = rom_rd(&device_handle, addr_version);
    
    let mut rom_name_array: [u8; 21] = [0; 21];
    for (index, item) in rom_name_array.iter_mut().enumerate() {
        *item = rom_rd(&device_handle, addr_rom_name + index as u16);
    }

    let rom_name = String::from_utf8_lossy(&rom_name_array).to_string();
    //let rom_name = str::from_utf8_unchecked(&rom_name_array).trim_end().to_string();
    //let rom_name = str::from_utf8_unchecked(&rom_name_array).trim_end().to_string();
    //let rom_name = str::from_utf8(&rom_name_array).expect("Invalid UTF-8 when parsing the ROM name").trim_end().to_string();

    let upper = (rom_rd(&device_handle, addr_compliment_check) as u16) << 8;
    let lower = rom_rd(&device_handle, addr_compliment_check + 1) as u16;
    let compliment_check = lower | upper;

    let upper: u16 = (rom_rd(&device_handle, addr_checksum) as u16) << 8;
    let lower: u16 = rom_rd(&device_handle, addr_checksum + 1) as u16;
    let checksum = lower | upper;

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
    //
    return SnesHeader { rom_name,
                        map_mode,
                        rom_type,
                        rom_size,
                        sram_size,
                        exp_ram_size,
                        destination_code,
                        developer_code,
                        version,
                        compliment_check,
                        checksum
    };
}

// Device functions

pub fn rom_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> u8 {
    let request = 4; // 4 is for SNES
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, request, SNES_ROM_RD, operand, 0);
    return buf[2];
}

pub fn set_bank<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) {
    let request = 4; // 4 is for SNES
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, SNES_SET_BANK, operand, 0);
}

fn match_developer(dev_code: u8) -> Option<&'static str> {
    match dev_code {
        0x01 => Some("Nintendo"),
        0x03 => Some("Imagineer-Zoom"),
        0x05 => Some("Zamuse"),
        0x06 => Some("Falcom"),
        0x08 => Some("Capcom"),
        0x09 => Some("HOT-B"),
        0x0a => Some("Jaleco"),
        0x0b => Some("Coconuts"),
        0x0c => Some("Rage Software"),
        0x0e => Some("Technos"),
        0x0f => Some("Mebio Software"),
        0x12 => Some("Gremlin Graphics"),
        0x13 => Some("Electronic Arts"),
        0x15 => Some("COBRA Team"),
        0x16 => Some("Human/Field"),
        0x17 => Some("KOEI"),
        0x18 => Some("Hudson Soft"),
        0x1a => Some("Yanoman"),
        0x1c => Some("Tecmo"),
        0x1e => Some("Open System"),
        0x1f => Some("Virgin Games"),
        0x20 => Some("KSS"),
        0x21 => Some("Sunsoft"),
        0x22 => Some("POW"),
        0x23 => Some("Micro World"),
        0x26 => Some("Enix"),
        0x27 => Some("Loriciel/Electro Brain"),
        0x28 => Some("Kemco"),
        0x29 => Some("Seta Co.,Ltd."),
        0x2d => Some("Visit Co.,Ltd."),
        0x31 => Some("Carrozzeria"),
        0x32 => Some("Dynamic"),
        0x33 => Some("Nintendo"),
        0x34 => Some("Magifact"),
        0x35 => Some("Hect"),
        0x3c => Some("Empire Software"),
        0x3d => Some("Loriciel"),
        0x40 => Some("Seika Corp."),
        0x41 => Some("UBI Soft"),
        0x46 => Some("System 3"),
        0x47 => Some("Spectrum Holobyte"),
        0x49 => Some("Irem"),
        0x4b => Some("Raya Systems/Sculptured Software"),
        0x4c => Some("Renovation Products"),
        0x4d => Some("Malibu Games/Black Pearl"),
        0x4f => Some("U.S. Gold"),
        0x50 => Some("Absolute Entertainment"),
        0x51 => Some("Acclaim"),
        0x52 => Some("Activision"),
        0x53 => Some("American Sammy"),
        0x54 => Some("GameTek"),
        0x55 => Some("Hi Tech Expressions"),
        0x56 => Some("LJN Toys"),
        0x5a => Some("Mindscape"),
        0x5d => Some("Tradewest"),
        0x5f => Some("American Softworks Corp."),
        0x60 => Some("Titus"),
        0x61 => Some("Virgin Interactive Entertainment"),
        0x62 => Some("Maxis"),
        0x67 => Some("Ocean"),
        0x69 => Some("Electronic Arts"),
        0x6b => Some("Laser Beam"),
        0x6e => Some("Elite"),
        0x6f => Some("Electro Brain"),
        0x70 => Some("Infogrames"),
        0x71 => Some("Interplay"),
        0x72 => Some("LucasArts"),
        0x73 => Some("Parker Brothers"),
        0x75 => Some("STORM"),
        0x78 => Some("THQ Software"),
        0x79 => Some("Accolade Inc."),
        0x7a => Some("Triffix Entertainment"),
        0x7c => Some("Microprose"),
        0x7f => Some("Kemco"),
        0x80 => Some("Misawa"),
        0x81 => Some("Teichio"),
        0x82 => Some("Namco Ltd."),
        0x83 => Some("Lozc"),
        0x84 => Some("Koei"),
        0x86 => Some("Tokuma Shoten Intermedia"),
        0x88 => Some("DATAM-Polystar"),
        0x8b => Some("Bullet-Proof Software"),
        0x8c => Some("Vic Tokai"),
        0x8e => Some("Character Soft"),
        0x8f => Some("I\"\"Max"),
        0x90 => Some("Takara"),
        0x91 => Some("CHUN Soft"),
        0x92 => Some("Video System Co., Ltd."),
        0x93 => Some("BEC"),
        0x95 => Some("Varie"),
        0x97 => Some("Kaneco"),
        0x99 => Some("Pack in Video"),
        0x9a => Some("Nichibutsu"),
        0x9b => Some("TECMO"),
        0x9c => Some("Imagineer Co."),
        0xa0 => Some("Telenet"),
        0xa4 => Some("Konami"),
        0xa5 => Some("K.Amusement Leasing Co."),
        0xa7 => Some("Takara"),
        0xa9 => Some("Technos Jap."),
        0xaa => Some("JVC"),
        0xac => Some("Toei Animation"),
        0xad => Some("Toho"),
        0xaf => Some("Namco Ltd."),
        0xb1 => Some("ASCII Co. Activison"),
        0xb2 => Some("BanDai America"),
        0xb4 => Some("Enix"),
        0xb6 => Some("Halken"),
        0xba => Some("Culture Brain"),
        0xbb => Some("Sunsoft"),
        0xbc => Some("Toshiba EMI"),
        0xbd => Some("Sony Imagesoft"),
        0xbf => Some("Sammy"),
        0xc0 => Some("Taito"),
        0xc2 => Some("Kemco"),
        0xc3 => Some("Square"),
        0xc4 => Some("Tokuma Soft"),
        0xc5 => Some("Data East"),
        0xc6 => Some("Tonkin House"),
        0xc8 => Some("KOEI"),
        0xca => Some("Konami USA"),
        0xcb => Some("NTVIC"),
        0xcd => Some("Meldac"),
        0xce => Some("Pony Canyon"),
        0xcf => Some("Sotsu Agency/Sunrise"),
        0xd0 => Some("Disco/Taito"),
        0xd1 => Some("Sofel"),
        0xd2 => Some("Quest Corp."),
        0xd3 => Some("Sigma"),
        0xd6 => Some("Naxat"),
        0xd8 => Some("Capcom Co., Ltd."),
        0xd9 => Some("Banpresto"),
        0xda => Some("Tomy"),
        0xdb => Some("Acclaim"),
        0xdd => Some("NCS"),
        0xde => Some("Human Entertainment"),
        0xdf => Some("Altron"),
        0xe0 => Some("Jaleco"),
        0xe2 => Some("Yutaka"),
        0xe4 => Some("T&ESoft"),
        0xe5 => Some("EPOCH Co.,Ltd."),
        0xe7 => Some("Athena"),
        0xe8 => Some("Asmik"),
        0xe9 => Some("Natsume"),
        0xea => Some("King Records"),
        0xeb => Some("Atlus"),
        0xec => Some("Sony Music Entertainment"),
        0xee => Some("IGS"),
        0xf1 => Some("Motown Software"),
        0xf2 => Some("Left Field Entertainment"),
        0xf3 => Some("Beam Software"),
        0xf4 => Some("Tec Magik"),
        0xf9 => Some("Cybersoft"),
        0xff => Some("Hudson Soft"),
        _ => None
    }
}

fn match_destination(dest_code: u8) -> Option<&'static str> {
    match dest_code {
        0 => Some("Japan (NTSC)"),
        1 => Some("USA (NTSC)"),
        2 => Some("Australia, Europe, Oceania and Asia (PAL)"),
        3 => Some("Sweden (PAL)"),
        4 => Some("Finland (PAL)"),
        5 => Some("Denmark (PAL)"),
        6 => Some("France (PAL)"),
        7 => Some("Holland (PAL)"),
        8 => Some("Spain (PAL)"),
        9 => Some("Germany, Austria and Switzerland (PAL)"),
        10 => Some("Italy (PAL)"),
        11 => Some("Hong Kong and China (PAL)"),
        12 => Some("Indonesia (PAL)"),
        13 => Some("Korea (PAL)"),
        _ => None
    }
}

fn match_hardware_type(rom_type: u8) -> Option<&'static str> {
    match rom_type {
        0x00 => Some("ROM Only"),
        0x01 => Some("ROM and RAM"),
        0x02 => Some("ROM and Save RAM"),
        0x03 => Some("ROM and DSP1"),
        0x13 => Some("ROM and SuperFX"),
        0x15 => Some("ROM and SuperFX and Save RAM"),
        0x1A => Some("ROM and SuperFX and Save RAM (Stunt Race FX)"),
        0x23 => Some("ROM and OBC1"),
        0x33 => Some("ROM and SA-1"),
        0x43 => Some("ROM and S-DD1"),
        0x45 => Some("ROM and S-DD1 and Save RAM"),
        0xF3 => Some("ROM and CX4"),
        0xF9 => Some("ROM and SPC7110 and RTC and Save RAM"),
        _ => None
    }
}

fn match_rom_upper_bound(rom_size: u8) -> Option<&'static str> {
    match rom_size {
        0x08 => Some("2 megabits"),
        0x09 => Some("4 megabits"),
        0x0A => Some("8 megabits"),
        0x0B => Some("16 megabits"),
        0x0C => Some("32 megabits"),
        0x0D => Some("64 megabits"),
        _ => None
    }
}

fn match_ram_size(ram_size: u8) -> Option<&'static str> {
    match ram_size {
        0x00 => Some("None"),
        0x01 => Some("16 kilobits"),
        0x02 => Some("32 kilobits"),
        0x03 => Some("64 kilobits"),
        0x05 => Some("256 kilobits"),
        0x06 => Some("512 kilobits"),
        _ => None
    }
}

fn match_map_mode(map_mode: u8) -> Option<&'static str> {
    match map_mode & 0xF {
        0x00 => Some("LoROM"),
        0x01 => Some("HiROM"),
        0x02 => Some("LoROM + S-DD1"),
        0x03 => Some("LoROM + SA-1"),
        0x05 => Some("ExHiROM"),
        0x0A => Some("HiROM + SPC7110"),
        _ => None
    }
}
