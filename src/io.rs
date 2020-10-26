use rusb::{DeviceHandle, UsbContext};

use crate::util;
use crate::opcodes::io::*;

pub fn reset<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, 2, IO_RESET, 0, 0);
}

pub fn nes_init<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, 2, NES_INIT, 0, 0);
}

pub fn exp0_pullup_test<T: UsbContext>(device_handle: &DeviceHandle<T>) -> u8 {
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, 2, EXP0_PULLUP_TEST, 0, 0);
    return buf[2];
}
