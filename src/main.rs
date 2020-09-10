extern crate libusb;

const VENDOR_ID: u16 = 0x16C0;
const PRODUCT_ID: u16 = 0x05DC;

const INL_MANUFACTURER: &str = "InfiniteNesLives.com";
const PROD: &str = "INL Retro-Prog";

fn main() {
    println!("Hello, world!");

    let mut context = libusb::Context::new().unwrap();

    println!("Checking");
    for mut device in context.devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        if (device_desc.vendor_id() == VENDOR_ID && device_desc.product_id() == PRODUCT_ID){
            println!("Found device");
            println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id());
        }
    }
}
