#![allow(dead_code)]

//set A16-23 aka bank number
pub const SNES_SET_BANK: u16 = 0x00;

//read from current bank at provided address
//SNES reset is unaffected
pub const SNES_ROM_RD: u16 = 0x01;	//RL=3

//write from current bank at provided address
//SNES reset is unaffected
pub const SNES_ROM_WR: u16 = 0x02;

pub const FLASH_WR_5V: u16 = 0x03;	//5v PLCC flash algo
pub const FLASH_WR_3V: u16 = 0x04;	//3v TSSOP flash algo

//similar to ROM RD/WR above, but /ROMSEL doesn't go low
pub const SNES_SYS_RD: u16 = 0x05;	//RL=3
pub const SNES_SYS_WR: u16 = 0x06;
