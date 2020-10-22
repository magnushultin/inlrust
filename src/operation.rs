use rusb::{DeviceHandle, UsbContext};

use crate::util;
use crate::opcodes::operation::*;

pub fn set_operation<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) {
    let request = 7; // 7 is for operation
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, SET_OPERATION, operand, 0);
}
/*
pub fn ppu_wr<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 1] = [0; 1];
    let value: u16 = (misc << 8) | NES_PPU_WR;
    util::read_device(device_handle, &mut buf, request, NES_PPU_WR, operand, value);
}
*/
