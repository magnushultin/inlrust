use rusb::{Context, DeviceHandle, Result, UsbContext, Version};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
#[allow(dead_code)]
mod bootload;
#[allow(dead_code)]
mod buffer;
#[allow(dead_code)]
mod io;
#[allow(dead_code)]
mod nes;
#[allow(dead_code)]
mod operation;
#[allow(dead_code)]
mod pinport;
mod util;

const VENDOR_ID: u16 = 0x16C0;
const PRODUCT_ID: u16 = 0x05DC;

const INL_MANUFACTURER: &str = "InfiniteNesLives.com";
const INL_PRODUCT: &str = "INL Retro-Prog";
const MIN_MAJOR_FW_VERSION: u8 = 2;

const MAX_VUSB: usize = 254;

const RETURN_ERR_IDX: usize = 0;
const RETURN_LEN_IDX: usize = 1;

// Since partial ord is not implemented for Version we do this ugly comparsion.
// TODO: Support minor and sub_minor minimum version
fn check_version(device_version: Version) -> bool {
    if device_version.major() >= MIN_MAJOR_FW_VERSION {
        if device_version.minor() > 0 {
            return true;
        }
        if device_version.sub_minor() > 0 {
            return true;
        }
    }
    return false;
}

fn main() {
    let context = Context::new().unwrap();
    let device_handle = get_device_handle(&context).unwrap();
    // get device version from firmware
    //"\x1b[0;31mSO\x1b[0m"
    println!("Get app version");
    bootload::get_app_ver(&device_handle);

    println!("IO_RESET");
    io::reset(&device_handle);
    // NES INIT
    println!("NES_INIT");
    io::nes_init(&device_handle);
    // TEST NROM
    test_nrom(&device_handle);

    // MIRROR
    //   detect_mapper_mirroring
    //   ciccom
    // READ
    let file = File::create("gamename.nes").unwrap();
    let mut f = BufWriter::new(file);

    //   create_header
    let mirroring = detect_mapper_mirroring(&device_handle).unwrap();
    create_header(&mut f, 32, 8, mirroring);
    //   dump_prgrom(file, 32, false)
    dump_prgrom(&device_handle, &mut f, 32);
    dump_chrrom(&device_handle, &mut f, 8);

    f.flush().unwrap();
    println!("IO_RESET");
    io::reset(&device_handle);
}

fn dump_prgrom<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u32,
) {
    let mut kb_per_read = 32;

    // Handle 16KB nroms.
    if rom_size_kb < kb_per_read {
        kb_per_read = rom_size_kb;
    }

    // local num_reads = rom_size_KB / KB_per_read
    let num_reads = rom_size_kb / kb_per_read;
    // local read_count = 0
    let mut read_count = 0;
    // local addr_base = 0x08	-- $8000
    let addr_base = 0x08;

    // while ( read_count < num_reads ) do
    //
    // 	if debug then print( "dump PRG part ", read_count, " of ", num_reads) end
    //  --               file   sizeKB       map         mem         debug
    // 	dump.dumptofile( file, KB_per_read, addr_base, "NESCPU_4KB", false )
    //
    // 	read_count = read_count + 1
    // end
    // NESCPU_4KB = 0x20
    while read_count < num_reads {
        dump(&device_handle, file, kb_per_read, addr_base, 0x20);
        read_count += 1;
    }
}

fn dump_chrrom<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u32,
) {
    let kb_per_read = 8;

    // local num_reads = rom_size_KB / KB_per_read
    let num_reads = rom_size_kb / kb_per_read;
    // local read_count = 0
    let mut read_count = 0;
    // local addr_base = 0x00	-- $0000
    let addr_base = 0x00;

    // while ( read_count < num_reads ) do
    // 	dump.dumptofile( file, KB_per_read, addr_base, "NESPPU_1KB", false )
    // 	read_count = read_count + 1
    // end
    // NESPPU_1KB = 0x21
    while read_count < num_reads {
        dump(&device_handle, file, kb_per_read, addr_base, 0x21);
        read_count += 1;
    }
}

fn dump<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    size_kb: u32,
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
            println!("DID NOT GET BUFF STATUS IN {} TRIES", try_nbr);
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
fn buffer_allocate<T: UsbContext>(
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

fn create_header<W: Write>(
    file: &mut BufWriter<W>,
    prg_size: u8,
    chr_size: u8,
    mirroring: Mirroring,
) {
    file.write(b"NES").unwrap();

    file.write_all(&[0x1A]).unwrap();
    // byte 4
    file.write_all(&[prg_size / 16]).unwrap();
    // byte 5
    file.write_all(&[chr_size / 8]).unwrap();

    // byte 6
    // NROM is mapper 0
    let mapper = 0;

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

fn test_nrom<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    println!("Testing NROM");
    println!("Detect mapper mirroring");
    detect_mapper_mirroring(&device_handle);
    //    IO EXP0_PULLUP_TEST
    io::exp0_pullup_test(&device_handle);
    //    read PRG-ROM manf ID
    nes::discrete_exp0_prgrom_wr(&device_handle, 0x5555, 0xAA);
    nes::discrete_exp0_prgrom_wr(&device_handle, 0x2AAA, 0x55);
    nes::discrete_exp0_prgrom_wr(&device_handle, 0x5555, 0x90);

    let rv = nes::cpu_rd(&device_handle, 0x8000);
    println!("PRG-ROM manf ID: 0x{:x}", rv);

    let rv = nes::cpu_rd(&device_handle, 0x8001);
    println!("PRG-ROM prod ID: 0x{:x}", rv);

    // Exit
    nes::discrete_exp0_prgrom_wr(&device_handle, 0x8000, 0xF0);

    //    read CHR-ROM manf ID
    nes::ppu_wr(&device_handle, 0x1555, 0xAA);
    nes::ppu_wr(&device_handle, 0x0AAA, 0x55);
    nes::ppu_wr(&device_handle, 0x1555, 0x90);

    let rv = nes::ppu_rd(&device_handle, 0x0000);
    println!("CHR-ROM manf ID: 0x{:x}", rv);

    let rv = nes::ppu_rd(&device_handle, 0x0001);
    println!("CHR-ROM prod ID: 0x{:x}", rv);
    // EXIT
    nes::ppu_wr(&device_handle, 0x0000, 0xF0);
}

#[derive(Eq, PartialEq)]
enum Mirroring {
    VERT,
    HORZ,
    SCNA,
    SCNB,
}

fn detect_mapper_mirroring<T: UsbContext>(device_handle: &DeviceHandle<T>) -> Result<Mirroring> {
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

fn get_device_handle<T: UsbContext>(context: &T) -> Option<DeviceHandle<T>> {
    println!("Checking");
    for device in context.devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        if device_desc.vendor_id() == VENDOR_ID && device_desc.product_id() == PRODUCT_ID {
            println!("Found device");
            println!(
                "Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id()
            );
            println!("Open device");
            let device_handle = device.open().unwrap();
            let product = device_handle
                .read_product_string_ascii(&device_desc)
                .unwrap();
            println!("Product string: {}", product);
            let manufacturer = device_handle
                .read_manufacturer_string_ascii(&device_desc)
                .unwrap();
            println!("Manufacturer string: {}", manufacturer);
            if manufacturer == INL_MANUFACTURER && product == INL_PRODUCT {
                let firmware_version = device_desc.device_version();
                if check_version(firmware_version) {
                    println!(
                        "INL retro-prog was found with firmware version {}.{}.{}",
                        firmware_version.major(),
                        firmware_version.minor(),
                        firmware_version.sub_minor()
                    );
                    return Some(device_handle);
                } else {
                    println!("INL Retro-Prog found, but firmware is too old!");
                }
            }
            break;
        }
    }
    return None;
}
