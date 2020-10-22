#![allow(dead_code)]
//opcodes
pub const CTL_ENABLE: u16 = 0;
pub const CTL_IP_PU: u16 = 1;
pub const CTL_IP_FL: u16 = 2;
pub const CTL_OP: u16 = 3;
pub const CTL_SET_LO: u16 = 4;
pub const CTL_SET_HI: u16 = 5;
pub const CTL_RD: u16 = 6; //RL=4	(error code, data length, LSB, MSB)
pub const CTL_OD: u16 = 24;
pub const CTL_PP: u16 = 25;

//operands

//PC0  "M2"	NES M2/phi signal
pub const C0: u16 = 0;
pub const M2: u16 = 0;
//PC1  "ROMSEL"	Cartridge rom enable
pub const C1: u16 = 1;
pub const ROMSEL: u16 = 1;
//PC2  "PRGRW"	NES CPU R/W signal
pub const C2: u16 = 2;
pub const PRGRW: u16 = 2;
//PC3  "FREE"	purple kazzo EXP flipflop latch, FREE on most AVR/adapter kazzos
pub const C3: u16 = 3;
pub const FREE: u16 = 3;
//PC4  "CSRD"	NES CHR/SNES /RD
pub const C4: u16 = 4;
pub const CSRD: u16 = 4;
//PC5  "CSWR"	NES CHR/SNES /WR
pub const C5: u16 = 5;
pub const CSWR: u16 = 5;
//PC6  "CICE" 	NES CIRAM /CE
pub const C6: u16 = 6;
pub const CICE: u16 = 6;
//PC7  "AHL" 	ADDR HI Latch
pub const C7: u16 = 7;
pub const AHL: u16 = 7;
//PC8  "EXP0" 	NES EXP0, cart-console /RESET
pub const C8: u16 = 8;
pub const EXP0: u16 = 8;
pub const SNES_RST: u16 = 8;
//PC9  "LED" 	kazzos tied this to NES EXP9, INL6 connects to CIC CLK
pub const C9: u16 = 9;
pub const LED: u16 = 9;
//PC10 "IRQ"	console CPU interrupt from cart
pub const C10: u16 = 10;
pub const IRQ: u16 = 10;
//PC11 "CIA10" 	NES CIRAM A10
pub const C11: u16 = 11;
pub const CIA10: u16 = 11;
//PC12 "BL" 	Bootloader pin
pub const C12: u16 = 12;
pub const BL: u16 = 12;
//PC13 "AXL" 	EXP FF latch and /OE, purple kazzos this was only /OE
pub const C13: u16 = 13;
pub const AXL: u16 = 13;
//INLretro6 adds following pins
//PC14 "AUDL"	cart audio
pub const C14: u16 = 14;
pub const AUDL: u16 = 14;
//PC15 "AUDR"	cart audio
pub const C15: u16 = 15;
pub const AUDR: u16 = 15;
//PC16 "GBP"	GB power selector
pub const C16: u16 = 16;
pub const GBP: u16 = 16;
//PC17 "SWD" 	mcu debug
pub const C17: u16 = 17;
pub const SWD: u16 = 17;
//PC18 "SWC" 	mcu debug
pub const C18: u16 = 18;
pub const SWC: u16 = 18;
//PC19 "AFL" 	flipflop addr expansion for FF0-7 (also CIC RESET on NES)
pub const C19: u16 = 19;
pub const AFL: u16 = 19;
//PC20 "COUT" CIC data out
pub const C20: u16 = 20;
pub const COUT: u16 = 20;
//PC21 "FCAPU" cart audio in
pub const C21: u16 = 21;
pub const FCAPU: u16 = 21;
//INLretro6 gains direct control over NES EXP port and is used for N64 control pins:
//PCxx "D8"
// pub const	Cxx: u16 =	xx
//PC22 "D9"
pub const C22: u16 = 22;
//PC23 "D10"
pub const C23: u16 = 23;
//PC24 "D11"
pub const C24: u16 = 24;
//PC25 "D12"
pub const C25: u16 = 25;
//PC26 "D13"
pub const C26: u16 = 26;
//PC27 "D14"
pub const C27: u16 = 27;

// D15 & D16 are defined as CICE/CIA10 above
pub const C28: u16 = 28;
pub const C29: u16 = 29;

//============================
//DATA PORT BYTE WIDE ACCESS
//opcode: type of pin operation
//operand: value to place on bus
//============================
pub const DATA_ENABLE: u16 = 7;
pub const DATA_IP_PU: u16 = 8;
pub const DATA_IP: u16 = 9;
pub const DATA_OP: u16 = 10;
pub const DATA_SET: u16 = 11;
pub const DATA_RD: u16 = 12; //RL=3 (error code, data length, databyte)

//============================
//ADDR PORT 16bit WIDE ACCESS
//opcode: type of operation
//operand: value to place on bus
//============================
pub const ADDR_ENABLE: u16 = 13;
pub const ADDR_PU: u16 = 14;
pub const ADDR_IP: u16 = 15;
pub const ADDR_OP: u16 = 16;
pub const ADDR_SET: u16 = 17;
pub const ADDR_RD: u16 = 26; //doesn't work on devices without direct access to 16bit address bus

//============================
//EXP PORT 8bit ACCESS (bits1-8)
//opcode: type of operation
//operand: value to place on bus
//============================
pub const EXP_ENABLE: u16 = 18;
pub const EXP_DISABLE: u16 = 19;
pub const EXP_SET: u16 = 20;

//============================
//HIGH ADDR PORT 8bit WIDE ACCESS
//opcode: type of operation
//operand: value to place on bus
//============================
pub const HADDR_ENABLE: u16 = 21;
pub const HADDR_DISABLE: u16 = 22;
pub const HADDR_SET: u16 = 23;

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
pub const FFADDR_ENABLE: u16 = 27;
pub const FFADDR_DISABLE: u16 = 28;
pub const FFADDR_SET: u16 = 29;
