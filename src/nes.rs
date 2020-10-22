use rusb::{DeviceHandle, UsbContext};

use crate::util;
use crate::opcodes::nes::*;

pub fn discrete_exp0_prgrom_wr<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    operand: u16,
    misc: u16,
) {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(
        device_handle,
        &mut buf,
        request,
        DISCRETE_EXP0_PRGROM_WR,
        operand,
        misc,
    );
}

pub fn cpu_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> u8 {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, request, NES_CPU_RD, operand, 0);
    return buf[2];
}

pub fn ppu_rd<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16) -> u8 {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, request, NES_PPU_RD, operand, 0);
    return buf[2];
}

pub fn ppu_wr<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 3; // 3 is for nes
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, NES_PPU_WR, operand, misc);
}
