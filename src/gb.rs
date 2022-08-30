use rusb::{DeviceHandle, UsbContext};
use std::str;

use crate::io;
use crate::util;
use crate::util::{CommandLineOptions};
use crate::opcodes::gb::*;


pub fn dump_gb<T: UsbContext>(device_handle: &DeviceHandle<T>, cmd_options: &CommandLineOptions) {
    io::reset(&device_handle);
    io::gameboy_init(&device_handle);
    io::gb_power_5v(&device_handle);

    let header = get_header(&device_handle);
    print_header(&header);
    

    io::reset(&device_handle);
}

#[derive(Debug)]
pub struct GbHeader {
    pub rom_name: String,     // 0134 - 0143 Title (013F - 0142 Manufacturer code in newer carts)
    pub developer_code: u8,   // 0144-0145 (newer carts), 014B older carts, 014B (old dev code) if 33h then use new 
    pub sgb_flag: u8,         // 0146 (00h No SGB, 03h SGB supported)
    pub cart_type: u8,        // 0147 Mapper/Cart type MBC1 ,MBC2 etc.
    pub rom_size: u8,         // 0148, Rom size (Typically calculated with 32KB shl N)
    pub ram_size: u8,         // 0149, Ram size if any
    pub dest_code: u8,        // 014A, sold in jap or elsewhere (00h jap, 01h non-jap)
    pub version: u8,          // 014C version number of game.
    pub header_checksum: u8,  // 014D Header checksum
    pub global_checksum: u16  // 014E-014F Global Checksum
}

fn get_header<T: UsbContext>(device_handle: &DeviceHandle<T>) -> GbHeader {

    let mut gb_header = GbHeader{
        rom_name: "".to_string(),
        developer_code:  0,
        sgb_flag: 0,
        cart_type: 0,
        rom_size:0,
        ram_size: 0,
        dest_code:0,
        version:0,
        header_checksum: 0,
        global_checksum: 0,
    };

    gb_header.sgb_flag = rom_rd(&device_handle, 0x146);
    gb_header.cart_type = rom_rd(&device_handle, 0x147);
    gb_header.rom_size = rom_rd(&device_handle, 0x148);
    gb_header.ram_size = rom_rd(&device_handle, 0x149);
    gb_header.dest_code = rom_rd(&device_handle, 0x14A);
    gb_header.version = rom_rd(&device_handle, 0x14C);
    gb_header.header_checksum = rom_rd(&device_handle, 0x14D);
    //  bytes
    let upper = (rom_rd(&device_handle, 0x14E) as u16) << 8;
    let lower = rom_rd(&device_handle, 0x14F) as u16;
    gb_header.global_checksum = lower | upper;

    // Get old dev code
    let old_dev = rom_rd(&device_handle, 0x14b);
    // If old dev is 0x33 then it is a newer type cart. Only newer types has the 16bytes names.

    let mut name_len = 11; // 16 is the full length, 11 for old type
    if old_dev == 0x33 { // new type
        name_len = 16;
        let up = ((rom_rd(&device_handle, 0x144) as u16) & 0xF) << 8;
        let low = rom_rd(&device_handle, 0x145) as u16 & 0xF;
        gb_header.developer_code = up as u8 | low as u8;
    } else {
        gb_header.developer_code = old_dev;
    }

    let mut rom_name = vec![0; name_len];
    for (index, item) in rom_name.iter_mut().enumerate() {
        *item = rom_rd(&device_handle, 0x134 + index as u16);
    }

    gb_header.rom_name = String::from_utf8_lossy(&rom_name).to_string();

    return gb_header;
}

fn print_header(header: &GbHeader) {
    println!("------------ HEADER ------------");
    println!("Name: {}", header.rom_name);
    println!("Developer: {}", match_developer(header.developer_code).unwrap_or("Unknown"));
    println!("Super Gameboy support: {}", if header.sgb_flag == 3 {"yes"} else {"no"});
    println!("Cart Type: {}", match_cart_type(header.cart_type).unwrap_or("Unknown"));
    println!("upper bound rom_size: {} KiB", 32 * (1 << header.rom_size)); // 32 KiB × (1 << <value>)
    println!("ram_size: {}", match_ram_size(header.ram_size).unwrap_or("Unknown"));
    println!("Destination: {}", if header.dest_code == 0 {"Jap (probably)"} else {"Overseas"});
    println!("Version: 0x{:X}", header.version);
    println!("Header checksum: {:X}", header.header_checksum);
    println!("Global checksum: {:X}", header.global_checksum);
    println!("--------------------------------");
}

fn match_ram_size(ram_size: u8) -> Option<&'static str> {
    match ram_size {
        0x00 => Some("None"),
        0x02 => Some("8 kilobits"),
        0x03 => Some("32 kilobits"),
        0x04 => Some("128 kilobits"),
        0x05 => Some("64 kilobits"),
        _ => None
    }
}

fn match_cart_type(cart_type: u8) -> Option<&'static str> {
    match cart_type {
        0x00 => Some("ROM Only"),
        0x01 => Some("MBC1"),
        0x02 => Some("MBC1+RAM"),
        0x03 => Some("MBC1+RAM+BATTERY"),
        0x05 => Some("MBC2"),
        0x06 => Some("MBC2+BATTERY"),
        0x08 => Some("ROM+RAM 1"),
        0x09 => Some("ROM+RAM+BATTERY 1"),
        0x0B => Some("MMM01"),
        0x0C => Some("MMM01+RAM"),
        0x0D => Some("MMM01+RAM+BATTERY"),
        0x0F => Some("MBC3+TIMER+BATTERY"),
        0x10 => Some("MBC3+TIMER+RAM+BATTERY 2"),
        0x11 => Some("MBC3"),
        0x12 => Some("MBC3+RAM 2"),
        0x13 => Some("MBC3+RAM+BATTERY 2"),
        0x19 => Some("MBC5"),
        0x1A => Some("MBC5+RAM"),
        0x1B => Some("MBC5+RAM+BATTERY"),
        0x1C => Some("MBC5+RUMBLE"),
        0x1D => Some("MBC5+RUMBLE+RAM"),
        0x1E => Some("MBC5+RUMBLE+RAM+BATTERY"),
        0x20 => Some("MBC6"),
        0x22 => Some("MBC7+SENSOR+RUMBLE+RAM+BATTERY"),
        0xFC => Some("POCKET CAMERA"),
        0xFD => Some("BANDAI TAMA5"),
        0xFE => Some("HuC3"),
        0xFF => Some("HuC1+RAM+BATTERY"),
        _ => None
    }
}

fn match_developer(dev_code: u8) -> Option<&'static str> {
    match dev_code {
        0x01 => Some("Nintendo"),
        0x08 => Some("Capcom"),
        0x09 => Some("Hot-B"),
        0x0A => Some("Jaleco"),
        0x0B => Some("Coconuts"),
        0x0C => Some("Elite Systems"),
        0x13 => Some("Electronic Arts"),
        0x18 => Some("Hudson Soft"),
        0x19 => Some("b-ai or itc entertainment"),
        0x1A => Some("Yanoman"),
        0x1D => Some("clary"),
        0x1F => Some("Virgin"),
        0x20 => Some("kss"),
        0x22 => Some("pow"),
        0x24 => Some("PCM Complete"),
        0x25 => Some("san-x"),
        0x28 => Some("Kemco Japan or kotobuki systems"),
        0x29 => Some("seta"),
        0x30 => Some("Viacom or Infogrames"),
        0x31 => Some("Nintendo"),
        0x32 => Some("Bandai"),
        0x33 => Some("Ocean/Acclaim"),
        0x34 => Some("Konami"),
        0x35 => Some("Hector"),
        0x37 => Some("Taito"),
        0x38 => Some("Hudson or Capcom"),
        0x39 => Some("Banpresto"),
        0x3C => Some("*entertainment i"),
        0x3E => Some("gremlin"),
        0x41 => Some("Ubi Soft"),
        0x42 => Some("Atlus"),
        0x44 => Some("Malibu"),
        0x46 => Some("angel"),
        0x47 => Some("Bullet-Proof or spectrum holoby"),
        0x49 => Some("irem"),
        0x4A => Some("virgin"), 
        0x4D => Some("malibu"), 
        0x4F => Some("u.s. gold"),
        0x50 => Some("Absolute"),
        0x51 => Some("Acclaim"),
        0x52 => Some("Activision"),
        0x53 => Some("American sammy"),
        0x54 => Some("Konami or Gametek"),
        0x55 => Some("Hi tech entertainment or Park Place"),
        0x56 => Some("LJN"),
        0x57 => Some("Matchbox"),
        0x58 => Some("Mattel"),
        0x59 => Some("Milton Bradley"),
        0x5A => Some("mindscape"),
        0x5B => Some("romstar"),
        0x5C => Some("naxat soft"),
        0x5D => Some("tradewest"),
        0x60 => Some("Titus"),
        0x61 => Some("Virgin"),
        0x64 => Some("LucasArts"),
        0x67 => Some("Ocean"),
        0x69 => Some("Electronic Arts"),
        0x6E => Some("Elite Systems"),
        0x6F => Some("Electro Brain"),
        0x70 => Some("Infogrames"),
        0x71 => Some("Interplay"),
        0x72 => Some("Broderbund"),
        0x73 => Some("sculptured"),
        0x75 => Some("the sales curve or sci"),
        0x78 => Some("THQ"),
        0x79 => Some("Accolade"),
        0x7A => Some("Triffix Entertainment"),
        0x7C => Some("Microprose"),
        0x7F => Some("Kemco"),
        0x80 => Some("misawa"),
        0x83 => Some("lozc"),
        0x86 => Some("Tokuma Shoten Intermedia"),
        0x87 => Some("Tsukuda Original"),
        0x8B => Some("bullet-proof software"),
        0x8C => Some("Vic Tokai"),
        0x8E => Some("Ape"),
        0x8F => Some("i'max"),
        0x91 => Some("Chunsoft"),
        0x92 => Some("Video system or Ocean/Acclaim"),
        0x93 => Some("tsuburava"),
        0x95 => Some("Varie"),
        0x96 => Some("Yonezawa/s’pal"),
        0x97 => Some("Kaneko"),
        0x99 => Some("Pack in soft or arc"),
        0x9A => Some("Nihon Bussan"),
        0x9B => Some("Tecmo"),
        0x9C => Some("Imagineer"),
        0x9D => Some("Banpresto"),
        0x9F => Some("Nova"),
        0xA1 => Some("Hori Electric"),
        0xA2 => Some("Bandai"),
        0xA4 => Some("Konami (Yu-Gi-Oh!)"),
        0xA6 => Some("kawada"),
        0xA7 => Some("takara"),
        0xA9 => Some("technos japan"),
        0xAA => Some("Broderbund"),
        0xAC => Some("Toei Animation"),
        0xAD => Some("Toho"),
        0xAF => Some("Namco"),
        0xB0 => Some("acclaim"),
        0xB1 => Some("ascii or nexoft"),
        0xB2 => Some("bandai"),
        0xB4 => Some("enix"),
        0xB6 => Some("hal"),
        0xB7 => Some("snk"),
        0xB9 => Some("pony canyon"),
        0xBA => Some("*culture brain o"),
        0xBB => Some("sunsoft"),
        0xBD => Some("sony imagesoft"),
        0xBF => Some("sammy"),
        0xC0 => Some("taito"),
        0xC2 => Some("kemco"),
        0xC3 => Some("squaresoft"),
        0xC4 => Some("*tokuma shoten i"),
        0xC5 => Some("data east"),
        0xC6 => Some("tonkin house"),
        0xC8 => Some("koei"),
        0xC9 => Some("ufl"),
        0xCA => Some("ultra"),
        0xCB => Some("vap"),
        0xCC => Some("use"),
        0xCD => Some("meldac"),
        0xCE => Some("*pony canyon or"),
        0xCF => Some("angel"),
        0xD0 => Some("taito"),
        0xD1 => Some("sofel"),
        0xD2 => Some("quest"),
        0xD3 => Some("sigma enterprises"),
        0xD4 => Some("ask kodansha"),
        0xD6 => Some("naxat soft"),
        0xD7 => Some("copya systems"),
        0xD9 => Some("banpresto"),
        0xDA => Some("tomy"),
        0xDB => Some("ljn"),
        0xDD => Some("ncs"),
        0xDE => Some("human"),
        0xDF => Some("altron"),
        0xE0 => Some("jaleco"),
        0xE1 => Some("towachiki"),
        0xE2 => Some("uutaka"),
        0xE3 => Some("varie"),
        0xE5 => Some("epoch"),
        0xE7 => Some("athena"),
        0xE8 => Some("asmik"),
        0xE9 => Some("natsume"),
        0xEA => Some("king records"),
        0xEB => Some("atlus"),
        0xEC => Some("epic/sony records"),
        0xEE => Some("igs"),
        0xF0 => Some("a wave"),
        0xF3 => Some("extreme entertainment"),
        0xFF => Some("ljn"),
        _ => None
    }
}


// Device functions

pub fn rom_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> u8 {
    let request = 12; // 12 is for GB
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, request, GAMEBOY_RD, operand, 0);
    return buf[2];
}