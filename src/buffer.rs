use rusb::{DeviceHandle, UsbContext};

use crate::util;
use crate::opcodes::buffer::*;


pub fn raw_buffer_reset<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, RAW_BUFFER_RESET, 0, 0);
}

pub fn allocate_buffer0<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(
        device_handle,
        &mut buf,
        request,
        ALLOCATE_BUFFER0,
        operand,
        misc,
    );
}

pub fn allocate_buffer1<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(
        device_handle,
        &mut buf,
        request,
        ALLOCATE_BUFFER1,
        operand,
        misc,
    );
}

pub fn set_reload_pagenum0<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    operand: u16,
    misc: u16,
) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(
        device_handle,
        &mut buf,
        request,
        SET_RELOAD_PAGENUM0,
        operand,
        misc,
    );
}

pub fn set_reload_pagenum1<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    operand: u16,
    misc: u16,
) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(
        device_handle,
        &mut buf,
        request,
        SET_RELOAD_PAGENUM1,
        operand,
        misc,
    );
}

pub fn set_mem_n_part<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(
        device_handle,
        &mut buf,
        request,
        SET_MEM_N_PART,
        operand,
        misc,
    );
}

pub fn set_map_n_mapvar<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(
        device_handle,
        &mut buf,
        request,
        SET_MAP_N_MAPVAR,
        operand,
        misc,
    );
}

pub fn get_cur_buff_status<T: UsbContext>(device_handle: &DeviceHandle<T>) -> u8 {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 3] = [0; 3];
    util::read_device(device_handle, &mut buf, request, GET_CUR_BUFF_STATUS, 0, 0);
    return buf[2];
}

pub fn buff_payload<T: UsbContext>(device_handle: &DeviceHandle<T>, buf: &mut [u8]) {
    let request = 5; // 5 is for buffer
    util::read_device_no_check(device_handle, buf, request, BUFF_PAYLOAD, 0, 0);
}
