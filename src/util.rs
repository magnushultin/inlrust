use rusb::{request_type, DeviceHandle, Direction, Recipient, RequestType, UsbContext};
use std::time::Duration;
use std::io::prelude::*;
use std::io::BufWriter;

use crate::buffer;
use crate::operation;

const RETURN_ERR_IDX: usize = 0;

const RED_START: &str = "\x1b[0;31m";
const COLOR_END: &str = "\x1b[0m";

pub fn dump<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    size_kb: u16,
    map: u16,
    mem: u16,
) {
    let buff0 = 0;
    let buff1 = 1;

    operation::set_operation(&device_handle, 0x01);

    buffer::raw_buffer_reset(&device_handle);

    buffer_allocate(&device_handle, 2, 128);

    buffer::set_mem_n_part(&device_handle, (mem << 8) | 0xDD, buff0);
    buffer::set_mem_n_part(&device_handle, (mem << 8) | 0xDD, buff1);

    buffer::set_map_n_mapvar(&device_handle, (map << 8) | 0, buff0);
    buffer::set_map_n_mapvar(&device_handle, (map << 8) | 0, buff1);

    operation::set_operation(&device_handle, 0xD2);

    let mut buf: [u8; 128] = [0; 128];
    let mut buff_status = 0;
    for _ in 0..((size_kb as u32) * 1024 / 128) {
        for try_nbr in 0..20 {
            buff_status = buffer::get_cur_buff_status(&device_handle);
            // DUMPED = 0xD8
            if buff_status == 0xD8 {
                break;
            }
            if try_nbr == 19 {
                println!("Did not get buff_status within 20 tries");
            }
        }
        if buff_status != 0xD8 {
            println!("DID NOT GET BUFF STATUS");
            println!("STOPPING!");
            break;
        }

        buffer::buff_payload(&device_handle, &mut buf);

        file.write_all(&buf).unwrap();
    }

    operation::set_operation(&device_handle, 0x01);
    buffer::raw_buffer_reset(&device_handle);
}

pub fn buffer_allocate<T: UsbContext>(
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
    buffer::allocate_buffer0(&device_handle, (buff0id << 8) | buff0basebank, numbanks);
    buffer::allocate_buffer1(&device_handle, (buff1id << 8) | buff1basebank, numbanks);
    buffer::set_reload_pagenum0(&device_handle, buff0_firstpage, reload);
    buffer::set_reload_pagenum1(&device_handle, buff1_firstpage, reload);
}

pub fn read_device<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    mut buf: &mut [u8],
    request: u8,
    opcode: u16,
    operand: u16,
    misc: u16,
) {
    let request_type = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let request = request; // 2 is for IO, 3 is for NES, 10 is bootload see shared_dictionaries.h
                           // let value: u16 = io::IO_RESET; // op_io[opcode] | misc << 8
    let value: u16 = (misc << 8) | opcode; // op_io[opcode] | misc << 8
    let index = operand; // operand 0 is default see Rlen in shared_io.h
                         // let mut buf: Vec<u8> = vec![0; size];
                         // let mut buf:[u8; 1]=[0; 1]; // no Rlen so 1
    let timeout = Duration::from_secs(1);

    let _ = device_handle
        .read_control(request_type, request, value, index, &mut buf, timeout)
        .unwrap();
    let error_code = buf[RETURN_ERR_IDX];
    // println!("{} bytes read", bytes);
    //TODO: return error if not error_code 0
    // TODO: Check returned bytes
    if error_code != 0 {
        println!("{}GOT ERROR{}: {}", RED_START, COLOR_END, error_code)
    }
}

pub fn read_device_no_check<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    mut buf: &mut [u8],
    request: u8,
    opcode: u16,
    operand: u16,
    misc: u16,
) {
    let request_type = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let request = request; // 2 is for IO, 3 is for NES, 10 is bootload see shared_dictionaries.h
                           // let value: u16 = io::IO_RESET; // op_io[opcode] | misc << 8
    let value: u16 = (misc << 8) | opcode; // op_io[opcode] | misc << 8
    let index = operand; // operand 0 is default see Rlen in shared_io.h
                         // let mut buf: Vec<u8> = vec![0; size];
                         // let mut buf:[u8; 1]=[0; 1]; // no Rlen so 1
    let timeout = Duration::from_secs(1);

    device_handle
        .read_control(request_type, request, value, index, &mut buf, timeout)
        .unwrap();
}

// Command line options
#[derive(Debug)]
pub struct CommandLineOptions {
    pub console: String,
    pub filename: String,
    pub savefile: String,
    pub mapper: String,
    pub prg_size: u16, // x
    pub chr_size: u16  // y
}

pub fn help() {
    println!("
Usage: program [options]

Options/Flags:
  --help, -h                                    Displays this message.
  -c console                                    Console port, (NES, SNES)
  -d filename                                   Dump cartridge ROMs to this filename
  -a filename                                   If provided, write ram to this filename
  -m mapper                                     NES:    (action53,bnrom,cdream,cninja,cnrom,dualport,easynsf,fme7,
                                                         mapper30,mmc1,mmc3,mmc4,mmc5,nrom,unrom)
  -x size_kbytes                                NES-only, size of PRG-ROM in kilobytes
  -y size_kbytes                                NES-only, size of CHR-ROM in kilobytes
  -w size_kbytes                                NES-only, size of WRAM in kilobytes
")
}

pub fn parse_command_line(args: &[String]) -> Result<CommandLineOptions, String> {
    if args.len() < 2 {
        return Err(String::from("Not enough arguments."))
    }
    let mut console = "".to_owned();
    let mut filename = "".to_owned();
    let mut savefile = "".to_owned();
    let mut mapper = "".to_owned();
    let mut prg_size = 0;
    let mut chr_size = 0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => return Err(String::from("")),
            "-c" =>  {
                console = args[i+1].clone();
                i += 1;
            },
            "-d" =>  {
                filename = args[i+1].clone();
                i += 1;
            },
            "-a" =>  {
                savefile = args[i+1].clone();
                i += 1;
            },
            "-m" =>  {
                mapper = args[i+1].clone();
                i += 1;
            },
            "-x" =>  {
                prg_size = parse_number(&args[i+1])?;
                i += 1;
            },
            "-y" =>  {
                chr_size = parse_number(&args[i+1])?;
                i += 1;
            },
            _ => (),
        }
        i += 1;
    }

    return Ok(CommandLineOptions { console, filename , savefile, mapper, prg_size, chr_size})
}

fn parse_number(argument: &String) -> Result<u16, String> {
    let input_opt = argument.clone().parse::<u16>();
    let size = match input_opt {
        Ok(size) => size,
        Err(e) => {
            return Err(format!("While parsing \"{}\" got err: {}", argument, e));
        }
    };
    return Ok(size);
}
