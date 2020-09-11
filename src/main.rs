use rusb::{Context, Device, DeviceHandle, Result, Version, UsbContext};

const VENDOR_ID: u16 = 0x16C0;
const PRODUCT_ID: u16 = 0x05DC;

const INL_MANUFACTURER: &str = "InfiniteNesLives.com";
const INL_PRODUCT: &str = "INL Retro-Prog";
const MIN_MAJOR_FW_VERSION: u8 = 2;


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
}
fn get_device_handle<T: UsbContext>(context: &T) -> Option<DeviceHandle<T>> {
    println!("Checking");
    for mut device in context.devices().unwrap().iter() {
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
