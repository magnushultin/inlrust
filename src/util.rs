use rusb::{request_type, DeviceHandle, Direction, Recipient, RequestType, UsbContext};
use std::time::Duration;

const RETURN_ERR_IDX: usize = 0;

const RED_START: &str = "\x1b[0;31m";
const COLOR_END: &str = "\x1b[0m";

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

    let bytes = device_handle
        .read_control(request_type, request, value, index, &mut buf, timeout)
        .unwrap();
    let error_code = buf[RETURN_ERR_IDX];
    // println!("{} bytes read", bytes);
    //TODO: return error if not error_code 0
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

    let bytes = device_handle
        .read_control(request_type, request, value, index, &mut buf, timeout)
        .unwrap();
}

// Command line options
#[derive(Debug)]
pub struct CommandLineOptions {
    pub console: String,
    pub filename: String,
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

    return Ok(CommandLineOptions { console, filename , mapper, prg_size, chr_size})
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
