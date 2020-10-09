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
