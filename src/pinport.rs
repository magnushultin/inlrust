use rusb::{DeviceHandle, Result, UsbContext};

use crate::util;
use crate::opcodes::pinport::*;

pub fn addr_set<T: UsbContext>(device_handle: &DeviceHandle<T>, address: u16) {
    let request = 1; // 1 is for pinport
    let mut buf: [u8; 1] = [0; 1]; // no Rlen so 1
    util::read_device(device_handle, &mut buf, request, ADDR_SET, address, 0);
}

pub fn ctl_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> Result<u16> {
    let mut buf: [u8; 4] = [0; 4];
    util::read_device(device_handle, &mut buf, 1, CTL_RD, operand, 0);
    let result: u16 = ((buf[3] as u16) << 8) | buf[2] as u16;
    return Ok(result);
}
