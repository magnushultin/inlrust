use rusb::{DeviceHandle, UsbContext};

use crate::util;

// OPCODES with no operand and no return value besides SUCCESS/ERROR_CODE

//Discrete board PRG-ROM only write, does not write to mapper
//This is a /WE controlled write with data latched on rising edge EXP0
//PRG-ROM /WE <- EXP0 w/PU
//PRG-ROM /OE <- /ROMSEL
//PRG-ROM /CE <- GND
//PRG-ROM write: /WE & /CE low, /OE high
//mapper '161 CLK  <- /ROMSEL
//mapper '161 /LOAD <- PRG R/W
//wValueMSB: data
//wIndex: address
const DISCRETE_EXP0_PRGROM_WR: u16 = 0x00;

const NES_PPU_WR: u16 = 0x01;

//generic CPU write with M2 toggle as expected with NES CPU
// A15 decoded to enable /ROMSEL as it should
const NES_CPU_WR: u16 = 0x02;

//const DISCRETE_EXP0_MAPPER_WR: u16 = 0x03;

//write to an MMC1 register, provide bank/address & data
const NES_MMC1_WR: u16 = 0x04;

const NES_DUALPORT_WR: u16 = 0x05;

const DISC_PUSH_EXP0_PRGROM_WR: u16 = 0x06;

const MMC3_PRG_FLASH_WR: u16 = 0x07; //TODO set return lengths for all these functions
const MMC3_CHR_FLASH_WR: u16 = 0x08;
const NROM_PRG_FLASH_WR: u16 = 0x09;
const NROM_CHR_FLASH_WR: u16 = 0x0A;
const CNROM_CHR_FLASH_WR: u16 = 0x0B; //needs cur_bank & bank_table prior to calling
const CDREAM_CHR_FLASH_WR: u16 = 0x0C; //needs cur_bank & bank_table prior to calling
const UNROM_PRG_FLASH_WR: u16 = 0x0D; //needs cur_bank & bank_table prior to calling
const MMC1_PRG_FLASH_WR: u16 = 0x0E;
const MMC1_CHR_FLASH_WR: u16 = 0x0F; //needs cur_bank set prior to calling
const MMC4_PRG_SOP_FLASH_WR: u16 = 0x10; //current bank must be selected, & needs cur_bank set prior to calling
const MMC4_CHR_FLASH_WR: u16 = 0x11; //needs cur_bank set prior to calling
const MAP30_PRG_FLASH_WR: u16 = 0x12; //needs cur_bank set prior to calling
const GTROM_PRG_FLASH_WR: u16 = 0x13; //desired bank must be selected
const MMC4_PRG_FLASH_WR: u16 = 0x14; //mapper mod to XOR A14 with A13

const SET_CUR_BANK: u16 = 0x20;
const SET_BANK_TABLE: u16 = 0x21;

const M2_LOW_WR: u16 = 0x22; //like CPU WR, but M2 stays low

//write a page worth of random data to ppu
//make sure the LSFR is initialized first in misc dict
//send start address in operand, doesn't have to be page boundary
//but A13 and /A13 get set once based on provided address.
const PPU_PAGE_WR_LFSR: u16 = 0x23;

const SET_NUM_PRG_BANKS: u16 = 0x24; //used for determining banktable structure for mapper 11 and such
const M2_HIGH_WR: u16 = 0x25; //like CPU WR, but M2 stays high
const FLASH_3V_WR: u16 = 0x25; //same as above but easier to remember when
                               //being used to write to 3v tssop flash
const MMC3S_PRG_FLASH_WR: u16 = 0x26; //TODO set return lengths for all these functions

//=============================================================================================
//	OPCODES WITH OPERAND AND RETURN VALUE plus SUCCESS/ERROR_CODE
//=============================================================================================

//read from NES CPU ADDRESS
//set /ROMSEL, M2, and PRG R/W
//read from cartridge just as NES's CPU would
//nice and slow trying to be more like the NES
const EMULATE_NES_CPU_RD: u16 = 0x80; //RL=3

//like the one above but not so slow..
const NES_CPU_RD: u16 = 0x81; //RL=3

const NES_PPU_RD: u16 = 0x82; //RL=3

//doesn't have operands just returns sensed CIRAM A10 mirroring
//now used to detect old firmware versions so NESmaker folks don't have to update firmware
const CIRAM_A10_MIRROR: u16 = 0x83; //RL=3
                                    ////returns VERT/HORIZ/1SCNA/1SCNB values:
                                    //	const	MIR_1SCNA	0x10
                                    //	const	MIR_1SCNB	0x11
                                    //	const	MIR_VERT	0x12
                                    //	const	MIR_HORZ	0x13

const NES_DUALPORT_RD: u16 = 0x84; //RL=3

const GET_CUR_BANK: u16 = 0x85; //RL=3
const GET_BANK_TABLE: u16 = 0x86; //RL=4 16bit value so 2 bytes need returned
const GET_NUM_PRG_BANKS: u16 = 0x87; //RL=3

const MMC5_PRG_RAM_WR: u16 = 0x88; //RL=3 Enable writting to PRG-RAM and then write a single byte
                                   //after written read back for verification as a timeout would cause fail

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
