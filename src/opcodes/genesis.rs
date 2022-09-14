#![allow(dead_code)]

//TODO THESE ARE JUST PLACE HOLDERS...
//oper=A1-15 update firmware address variable for FLASH_WR_ADDROFF use on subsequent calls
pub const GEN_SET_ADDR: u16 = 0;
//oper=A1-A16 C_CE & C_OE go low (update firmware address var ie GEN_SET_ADDR)
pub const GEN_ROM_RD: u16 = 1;	//RL=4 return error code, data len = 1, 2 byte of data (16bit word )

// GENESIS ADDR A17-23 along with #LO_MEM & #TIME
// TODO separate #LO_MEM & #TIME, they're currently fixed high
pub const GEN_SET_BANK: u16 = 2;