use rusb::{DeviceHandle, UsbContext};

use crate::util;

//BOOTLOAD opcodes

const LOAD_ADDRH: u16 = 2; //upper address half word used for various functions
const JUMP_ADDR: u16 = 3; //jump to address upper 16bit provided previous opcode

const PREP_FWUPDATE: u16 = 4; //leave main application and sets up for fwupdate

//operand provides 16bit value for RD/WR commands below
const SET_PTR_HI: u16 = 5;
const SET_PTR_LO: u16 = 6;
const GET_PTR: u16 = 7; //RL=6

//ALL OFFSETS ARE INTERPRETED AT POSITIVE UNSIGNED!
//read 16bit value from memory location being pointed to
//operand provides offset from current pointer, but doesn't modify the pointer
const RD_PTR_OFFSET: u16 = 8; //RL=4  0-error, 1-len, 2-LSB, 3-MSB
                              //operand provides 16bit value to be written, miscdata provides offset
const WR_PTR_OFFSET: u16 = 9;

//operand provides 16bit offset which is added to ptr before access
//then reads from that address
const RD_PTR_OFF_UP: u16 = 10; //RL=4  0-error, 1-len, 2-LSB, 3-MSB

//miscdata provide 8bit offset which is added to ptr before access
//operand is the 16bit value which is written to memory location being pointed to
const WR_PTR_OFF_UP: u16 = 11;

//application code version
//this is updated more frequently than the USB firmware version
const GET_APP_VER: u16 = 12; //RL=3  0-error, 1-len, 2-version  (2345-version if string "AV##")
                             //STM32 just set pointer to 0x08000800 and read 4 bytes for now
                             //AVR has to use this method..

//APPLICATION VERSION NUMBERS
//#define	APP_VERSION	"AV00"	//released with usb firmware v2.3
//main update was addition of usb firmware updater
//also added the bootloader pointer memory access
//include ram functions & starting to have NES flash algos return data
const APP_VERSION: u8 = 3; //released 8/16 with N64 dump fix and MMC2/4 support

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
