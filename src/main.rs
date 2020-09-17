use rusb::{Context, DeviceHandle, Result, Version, UsbContext, Direction, RequestType, Recipient, request_type};
use std::time::Duration;
mod io;

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
    println!("Get app version");
    get_app_version(&device_handle);

    println!("IO_RESET");
    io_reset(&device_handle);
    // NES INIT
    println!("NES_INIT");
    io_nes_init(&device_handle);
    // TEST NROM
    println!("Testing NROM");
    println!("Detect mapper mirroring");
    detect_mapper_mirroring(&device_handle);
    //    IO EXP0_PULLUP_TEST
    io_exp0_pullup_test(&device_handle);
    //    read PRG-ROM manf ID
    //    read CHR-ROM manf ID
    // MIRROR
    //   detect_mapper_mirroring
    //   ciccom
    // READ
    //   create_header
    //   dump_prgrom
    //   dump_chrrom
    // IO.RESET
    println!("IO_RESET");
    io_reset(&device_handle);
}

fn io_exp0_pullup_test<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    //   Wlen 1 is default
    //   operand 0 is default
    //   request is 2 for IO commands 3 for NES commands
    
    //    ep,	      dictionary  wValue[misc:opcode]             wIndex	wLength	 		data
    //direction.IN,               misc << 8 | io::IO_RESET        operand       data
    //count, data = usb_vend_xfr( 
    //        ep, dict["DICT_IO"], ( misc<<8 | op_io[opcode]),	operand,	wLength,	data)
    //

    let request_type = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let request = 2; // 2 is for IO, 3 is for NES, 10 is bootload see shared_dictionaries.h
    // let value: u16 = io::IO_RESET; // op_io[opcode] | misc << 8
    let value: u16 = io::EXP0_PULLUP_TEST; // op_io[opcode] | misc << 8
    let index = 0; // operand 0 is default see Rlen in shared_io.h
    let mut buf:[u8; 3]=[0; 3]; // RL 3
    let timeout = Duration::from_secs(1);

    let bytes = device_handle.read_control(request_type, request, value, index, &mut buf, timeout).unwrap();
    let error_code = buf[RETURN_ERR_IDX];
    println!("{} bytes read", bytes);
    //TODO: return error if not error_code 0
    if error_code != 0 {
        println!("GOT ERROR: {}", error_code)
    }
    println!("{} bytes read", bytes);
    println!("data len {}", buf[RETURN_LEN_IDX]);
    println!("EXP0 pull-up test: {:?}", &buf[RETURN_LEN_IDX + 1..]);
}

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
    pinport_addr_set(&device_handle, 0x0800);
    // readH = PINPORT CTL_RD, CIA10         RL=4 (err_code, data_len, LSB, MSB)
    //           1       6       11
    let read_h = pinport_ctl_rd(&device_handle, 11).unwrap();
    println!("Read h: {}", read_h);

    // PINPORT ADDR_SET, 0x0400
    println!("PINPORT_ADDR_SET 0x0400");
    pinport_addr_set(&device_handle, 0x0400);
    // readH = PINPORT CTL_RD, CIA10         RL=4 (err_code, data_len, LSB, MSB)
    //           1       6       11
    let read_v = pinport_ctl_rd(&device_handle, 11).unwrap();

    if read_v == 0 && read_h == 0 {
        println!("1SCNA - 1screen A mirroring");
        return Ok(Mirroring::SCNA);
    } else if read_v !=0 && read_h == 0 {
        println!("VERT - Vertical mirroring");
        return Ok(Mirroring::VERT);
    } else if read_v ==0 && read_h != 0 {
        println!("HORZ - Horizontal mirroring");
        return Ok(Mirroring::HORZ);
    } else {
        println!("1SCNB - 1screen B mirroring");
        return Ok(Mirroring::SCNB);
    };
}

fn pinport_ctl_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> Result<u16> {
    let request_type = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let request = 1; // 1 is for pinport
    let value: u16 = 6; // 17 is CTL_RD
    let index = operand; // CIA is 11
    let mut buf:[u8; 4]=[0; 4]; // RL=4
    let timeout = Duration::from_secs(1);

    let bytes = device_handle.read_control(request_type, request, value, index, &mut buf, timeout).unwrap();
    let error_code = buf[RETURN_ERR_IDX];
    println!("{} bytes read", bytes);
    if error_code != 0 {
        println!("GOT ERROR: {}", error_code)
    }

    println!("{} bytes read", bytes);
    println!("data len {}", buf[RETURN_LEN_IDX]);
    println!("CTL_RD: {:x?}", &buf);
    let result: u16 = ((buf[3] as u16) << 8) | buf[2] as u16;
    return Ok(result)
}

fn pinport_addr_set<T: UsbContext>(device_handle: &DeviceHandle<T>, address: u16) {
    let request_type = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let request = 1; // 1 is for pinport
    let value: u16 = 0x11; // 17 is addr set
    let index = address;
    let mut buf:[u8; 1]=[0; 1]; // no Rlen so 1
    let timeout = Duration::from_secs(1);

    let bytes = device_handle.read_control(request_type, request, value, index, &mut buf, timeout).unwrap();
    let error_code = buf[RETURN_ERR_IDX];
    println!("{} bytes read", bytes);
    if error_code != 0 {
        println!("GOT ERROR: {}", error_code)
    }
}

fn io_reset<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    //   Wlen 1 is default
    //   operand 0 is default
    //   request is 2 for IO commands 3 for NES commands
    
    //    ep,	      dictionary  wValue[misc:opcode]             wIndex	wLength	 		data
    //direction.IN,               misc << 8 | io::IO_RESET        operand       data
    //count, data = usb_vend_xfr( 
    //        ep, dict["DICT_IO"], ( misc<<8 | op_io[opcode]),	operand,	wLength,	data)
    //

    let request_type = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let request = 2; // 2 is for IO, 3 is for NES, 10 is bootload see shared_dictionaries.h
    // let value: u16 = io::IO_RESET; // op_io[opcode] | misc << 8
    let value: u16 = io::IO_RESET; // op_io[opcode] | misc << 8
    let index = 0; // operand 0 is default see Rlen in shared_io.h
    let mut buf:[u8; 1]=[0; 1]; // no Rlen so 1
    let timeout = Duration::from_secs(1);

    let bytes = device_handle.read_control(request_type, request, value, index, &mut buf, timeout).unwrap();
    let error_code = buf[RETURN_ERR_IDX];
    println!("{} bytes read", bytes);
    //TODO: return error if not error_code 0
    if error_code != 0 {
        println!("GOT ERROR: {}", error_code)
    }
}

fn io_nes_init<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let request_type = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let request = 2; // 2 is for IO, 3 is for NES, 10 is bootload see shared_dictionaries.h
    // let value: u16 = io::IO_RESET; // op_io[opcode] | misc << 8
    let value: u16 = io::NES_INIT; // op_io[opcode] | misc << 8
    let index = 0; // operand 0 is default see Rlen in shared_io.h
    let mut buf:[u8; 1]=[0; 1]; // no Rlen so 1
    let timeout = Duration::from_secs(1);

    let bytes = device_handle.read_control(request_type, request, value, index, &mut buf, timeout).unwrap();
    let error_code = buf[RETURN_ERR_IDX];
    println!("{} bytes read", bytes);
    //TODO: return error if not error_code 0
    if error_code != 0 {
        println!("GOT ERROR: {}", error_code)
    }
}

fn get_app_version<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let rtype = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let request = 10; // 2 is for IO, 3 is for NES, 10 is bootload see shared_dictionaries.h
    // let value: u16 = io::IO_RESET; // op_io[opcode] | misc << 8
    let value: u16 = 12; // op_io[opcode] | misc << 8
    let index = 0; // operand 0 is default in shared_io.h
    let mut buf:[u8; 3]=[0; 3]; // RLEN is 3
    let timeout = Duration::from_secs(1);

    let bytes = device_handle.read_control(rtype, request, value, index, &mut buf, timeout).unwrap();
    let error_code = buf[RETURN_ERR_IDX];
    println!("{} bytes read", bytes);
    println!("data len {}", buf[RETURN_LEN_IDX]);
    println!("firmware app ver request: {:x?}", &buf[2..]);
    //TODO: return error if not error_code 0
    if error_code != 0 {
        println!("GOT ERROR: {}", error_code)
    }
    //TODO: return error if data length does not match buffer length - error index and length index
}

fn get_device_handle<T: UsbContext>(context: &T) -> Option<DeviceHandle<T>> {
    println!("Checking");
    for device in context.devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        if device_desc.vendor_id() == VENDOR_ID && device_desc.product_id() == PRODUCT_ID {
            println!("Found device");
            println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id());
            println!("Open device");
            let device_handle = device.open().unwrap();
            let product = device_handle.read_product_string_ascii(&device_desc).unwrap();
            println!("Product string: {}", product);
            let manufacturer = device_handle.read_manufacturer_string_ascii(&device_desc).unwrap();
            println!("Manufacturer string: {}", manufacturer);
            if manufacturer == INL_MANUFACTURER && product == INL_PRODUCT {

                let firmware_version = device_desc.device_version();
                if check_version(firmware_version) {
                    println!("INL retro-prog was found with firmware version {}.{}.{}",
                        firmware_version.major(),firmware_version.minor(), firmware_version.sub_minor());
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
