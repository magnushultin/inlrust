use rusb::{Context, DeviceHandle, UsbContext, Version};
use std::env;
use std::process;
mod bootload;
mod buffer;
mod io;
mod nes;
mod snes;
mod operation;
mod pinport;
mod opcodes;
mod util;
mod nes_mappers;

const VENDOR_ID: u16 = 0x16C0;
const PRODUCT_ID: u16 = 0x05DC;

const INL_MANUFACTURER: &str = "InfiniteNesLives.com";
const INL_PRODUCT: &str = "INL Retro-Prog";
const MIN_MAJOR_FW_VERSION: u8 = 2;

fn main() {
    let args: Vec<String> = env::args().collect();

    let cmd_options = util::parse_command_line(&args).unwrap_or_else(|err| {
        println!("{}", err);
        util::help();
        process::exit(1);
    });
    println!("{:?}", cmd_options);

    let context = Context::new().unwrap();
    let device_handle = get_device_handle(&context).unwrap();
    // get device version from firmware
    println!("Get app version");
    bootload::get_app_ver(&device_handle);
    
    if cmd_options.console.to_lowercase() == "nes" {
        nes::dump_nes(&device_handle, &cmd_options);
    }
    else if cmd_options.console.to_lowercase() == "snes" {
        snes::dump_snes(&device_handle, &cmd_options);
    } else {
        println!("Console {} is not supported!", cmd_options.console);
    }

    io::reset(&device_handle);
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
