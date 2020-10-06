use rusb::{DeviceHandle, Result, UsbContext};

use crate::util;

//opcodes
const CTL_ENABLE: u16 = 0;
const CTL_IP_PU: u16 = 1;
const CTL_IP_FL: u16 = 2;
const CTL_OP: u16 = 3;
const CTL_SET_LO: u16 = 4;
const CTL_SET_HI: u16 = 5;
const CTL_RD: u16 = 6; //RL=4	(error code, data length, LSB, MSB)
const CTL_OD: u16 = 24;
const CTL_PP: u16 = 25;

//operands

//PC0  "M2"	NES M2/phi signal
const C0: u16 = 0;
const M2: u16 = 0;
//PC1  "ROMSEL"	Cartridge rom enable
const C1: u16 = 1;
const ROMSEL: u16 = 1;
//PC2  "PRGRW"	NES CPU R/W signal
const C2: u16 = 2;
const PRGRW: u16 = 2;
//PC3  "FREE"	purple kazzo EXP flipflop latch, FREE on most AVR/adapter kazzos
const C3: u16 = 3;
const FREE: u16 = 3;
//PC4  "CSRD"	NES CHR/SNES /RD
const C4: u16 = 4;
const CSRD: u16 = 4;
//PC5  "CSWR"	NES CHR/SNES /WR
const C5: u16 = 5;
const CSWR: u16 = 5;
//PC6  "CICE" 	NES CIRAM /CE
const C6: u16 = 6;
const CICE: u16 = 6;
//PC7  "AHL" 	ADDR HI Latch
const C7: u16 = 7;
const AHL: u16 = 7;
//PC8  "EXP0" 	NES EXP0, cart-console /RESET
const C8: u16 = 8;
const EXP0: u16 = 8;
const SNES_RST: u16 = 8;
//PC9  "LED" 	kazzos tied this to NES EXP9, INL6 connects to CIC CLK
const C9: u16 = 9;
const LED: u16 = 9;
//PC10 "IRQ"	console CPU interrupt from cart
const C10: u16 = 10;
const IRQ: u16 = 10;
//PC11 "CIA10" 	NES CIRAM A10
const C11: u16 = 11;
const CIA10: u16 = 11;
//PC12 "BL" 	Bootloader pin
const C12: u16 = 12;
const BL: u16 = 12;
//PC13 "AXL" 	EXP FF latch and /OE, purple kazzos this was only /OE
const C13: u16 = 13;
const AXL: u16 = 13;
//INLretro6 adds following pins
//PC14 "AUDL"	cart audio
const C14: u16 = 14;
const AUDL: u16 = 14;
//PC15 "AUDR"	cart audio
const C15: u16 = 15;
const AUDR: u16 = 15;
//PC16 "GBP"	GB power selector
const C16: u16 = 16;
const GBP: u16 = 16;
//PC17 "SWD" 	mcu debug
const C17: u16 = 17;
const SWD: u16 = 17;
//PC18 "SWC" 	mcu debug
const C18: u16 = 18;
const SWC: u16 = 18;
//PC19 "AFL" 	flipflop addr expansion for FF0-7 (also CIC RESET on NES)
const C19: u16 = 19;
const AFL: u16 = 19;
//PC20 "COUT" CIC data out
const C20: u16 = 20;
const COUT: u16 = 20;
//PC21 "FCAPU" cart audio in
const C21: u16 = 21;
const FCAPU: u16 = 21;
//INLretro6 gains direct control over NES EXP port and is used for N64 control pins:
//PCxx "D8"
// const	Cxx: u16 =	xx
//PC22 "D9"
const C22: u16 = 22;
//PC23 "D10"
const C23: u16 = 23;
//PC24 "D11"
const C24: u16 = 24;
//PC25 "D12"
const C25: u16 = 25;
//PC26 "D13"
const C26: u16 = 26;
//PC27 "D14"
const C27: u16 = 27;

// D15 & D16 are defined as CICE/CIA10 above
const C28: u16 = 28;
const C29: u16 = 29;

//============================
//DATA PORT BYTE WIDE ACCESS
//opcode: type of pin operation
//operand: value to place on bus
//============================
const DATA_ENABLE: u16 = 7;
const DATA_IP_PU: u16 = 8;
const DATA_IP: u16 = 9;
const DATA_OP: u16 = 10;
const DATA_SET: u16 = 11;
const DATA_RD: u16 = 12; //RL=3 (error code, data length, databyte)

//============================
//ADDR PORT 16bit WIDE ACCESS
//opcode: type of operation
//operand: value to place on bus
//============================
const ADDR_ENABLE: u16 = 13;
const ADDR_PU: u16 = 14;
const ADDR_IP: u16 = 15;
const ADDR_OP: u16 = 16;
const ADDR_SET: u16 = 17;
const ADDR_RD: u16 = 26; //doesn't work on devices without direct access to 16bit address bus

//============================
//EXP PORT 8bit ACCESS (bits1-8)
//opcode: type of operation
//operand: value to place on bus
//============================
const EXP_ENABLE: u16 = 18;
const EXP_DISABLE: u16 = 19;
const EXP_SET: u16 = 20;

//============================
//HIGH ADDR PORT 8bit WIDE ACCESS
//opcode: type of operation
//operand: value to place on bus
//============================
const HADDR_ENABLE: u16 = 21;
const HADDR_DISABLE: u16 = 22;
const HADDR_SET: u16 = 23;

//	CTL_OD: u16 =	24 above
//	CTL_PP: u16 =	25 above
//	ADDR_RD: u16 =26 above

//============================
//FLIPFLOP ADDR PORT 8bit WIDE ACCESS
//SEGA: FF0-7 connecto to A17-18, #AS, A20-23, #TIME
//opcode: type of operation
//operand: value to place on bus
//NOTE: these operations corrupt the ADDR bus, so call this first
//============================
const FFADDR_ENABLE: u16 = 27;
const FFADDR_DISABLE: u16 = 28;
const FFADDR_SET: u16 = 29;

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
