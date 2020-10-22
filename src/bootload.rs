use rusb::{DeviceHandle, UsbContext};

use crate::util;
use crate::opcodes::bootload::*;

pub fn get_app_ver<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, 10, GET_APP_VER, 0, 0);
    let app_version = buf[2];
    println!("App version is: {}", app_version);
    // TODO: Send back error for different app_versions
    if APP_VERSION != app_version {
        println!(
            "ERROR got the wrong app version. Should be {} but got {}",
            APP_VERSION, app_version
        );
    }
}
