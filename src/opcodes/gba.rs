#![allow(dead_code)]

//must have latched the address first
//rom will auto increment so can just call this repeatedly to read a sequence of addresses
pub const GBA_RD: u16 = 0x00;	//RL=4  return error code, data len = 1, 2 bytes of data

//operand A0-15, miscdata A16-23
//leaves /CE low for subsequent accesses
//leaves A16-23 as output
//leaves AD0-15 as input
pub const GBA_LATCH_ADDR: u16 = 0x02;

//take /CE high to finish above access
//put A16-23 back to input
pub const GBA_RELEASE_BUS: u16 = 0x03;