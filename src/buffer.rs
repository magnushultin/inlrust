use rusb::{DeviceHandle, UsbContext};

use crate::util;

//raw buffer banks & size
//This determines the raw buffer sram space on avr at firmware compile time
//one byte per bank is instantiated to keep track of that banks' allocation status
//a buffer must be as large as one bank, but a buffer can consume multiple banks
//only limit to buffer size is the buffer structure
//current buffer structure utilizes a single byte for current byte counter
//which limits to 256 bytes per buffer currently
//having 16bit value support would expand this, or somehow shifting current byte
//to account for multiple bytes could expand further
//const NUM_RAW_BANKS   8	// 8*32 = 256 bytes of buffer
const NUM_RAW_BANKS: u16 = 16; //16*32 = 512 bytes of buffer
                               //const NUM_RAW_BANKS   24	//24*32 = 768 DAMN THE TORPEDOS FULL SPEED AHEAD!!!

const RAW_BANK_SIZE: u16 = 32; //bank size in bytes

//number of buffer objects
//This controls number of static buffer objects instantiated in firmware at compile time
//note this also controls opcodes created/supported by firmware
//reducing number of buffers frees firmware sram by ~16bytes per buffer object
//not much reason to have less than 2 atleast allow double buffering
//so one can be getting loaded/unloaded by USB while other is dumping/flashing
//current max is 8, but only really limited by opcode definitions to address all buffers
//makes #ifdef code simpler to only allow buffer numbers that are power of 2
//const NUM_BUFFERS_2	2
const NUM_BUFFERS_4: u16 = 4;
//const NUM_BUFFERS_8	8

//=============================================================================================
//	OPCODES with up to 24bit operand and optional return value  besides SUCCESS/ERROR_CODE
//	PAYLOAD options listed as well
//=============================================================================================
//	Detect this opcode/operand setup with opcode between the following defines:
//
//------------------------------------
const BUFF_OPCODE_NRV_MIN: u16 = 0x00;
//opcodes in this range have NO RETURN besides error code and DON'T contain buff# in miscdata byte
//			----------------------------
const BUFFN_INMISC_MIN: u16 = 0x30; //NOTE OVERLAP!!
                                    //opcodes in this range have NO RETURN besides error code and DO contain buff# in miscdata byte
const BUFF_OPCODE_NRV_MAX: u16 = 0x3F;
//------------------------------------
const BUFF_PAYLOADN_MIN: u16 = 0x40;
//opcodes in this range are PAYLOADS and DO contain buff# in miscdata byte
const BUFF_PAYLOADN_MAX: u16 = 0x4F;
//------------------------------------
const BUFF_OPCODE_RV_MIN: u16 = 0x50;
//opcodes in this range HAVE RETURN besides error code and DO contain buff# in miscdata byte
const BUFFN_INMISC_MAX: u16 = 0x5F; //NOTE OVERLAP!!
                                    //			----------------------------
                                    //opcodes in this range HAVE RETURN value plus error code and DON'T contain buff# in miscdata byte
const BUFF_OPCODE_RV_MAX: u16 = 0x6F;
//------------------------------------
const BUFF_PAYLOAD_MIN: u16 = 0x70;
//opcodes in this range are PAYLOADS and DO NOT contain buff# in miscdata byte
const BUFF_PAYLOAD_MAX: u16 = 0x7F;
//=============================================================================================
//=============================================================================================

//------------------------------------------------------------------------------------------------
//opcodes in this range have NO RETURN besides error code and DON'T contain buff# in miscdata byte
//const BUFF_OPCODE_NRV_MIN: u16 = 0x00-2F
//------------------------------------------------------------------------------------------------

//blindly clear all allocation of raw buffer space
//reset all buffers to unallocated
//no operands no return value
const RAW_BUFFER_RESET: u16 = 0x00;

//------------------------------------------------------------------------------------------------
//opcodes in this range have NO RETURN besides error code and DO contain buff# in miscdata byte
//const BUFFN_INMISC_MIN: u16 = 0x30-3F	//NOTE OVERLAP!!
//------------------------------------------------------------------------------------------------
//SET BUFFER ELEMENTS

//memory type and part number
//miscdata: buffer number
//operMSB: memory type
//operLSB: part number
const SET_MEM_N_PART: u16 = 0x30;
//operand MSB memtype
const PRGROM: u16 = 0x10;
const CHRROM: u16 = 0x11;
const PRGRAM: u16 = 0x12;
const SNESROM: u16 = 0x13;
const SNESRAM: u16 = 0x14;
const GENESISROM: u16 = 0x15;

//Read specific sections of memory map
// 4KB/1KB naming designates the granularity of the starting address
// Any amount can be read, but unexpected behavior will result when reading past memory map limits
// designate the address base with mapper since this read is mapper independent
const NESCPU_4KB: u16 = 0x20; //mapper (bits 3-0) specifies A12-15 (4bits)
const NESPPU_1KB: u16 = 0x21; //mapper (bits 5-2) specifies A10-13 (4bits)
                              //DON'T WANT TO USE THESE ANY MORE, USE THE PAGE VERSIONS BELOW

//since the types above only specify the granularity of the read, there is no reason
//to limit it to 1-4KByte.  May as well give page granularity and use the whole mapper byte!
const NESCPU_PAGE: u16 = 0x22; //mapper byte specifies A15-8
const NESPPU_PAGE: u16 = 0x23; //mapper byte specifies A13-8	 bits 6 & 7 can't be set
const SNESROM_PAGE: u16 = 0x24; //mapper byte specifies A15-8 ROMSEL low
const SNESSYS_PAGE: u16 = 0x25; //mapper byte specifies A15-8 ROMSEL high
const GAMEBOY_PAGE: u16 = 0x26; //mapper byte specifies A15-8
const GBA_ROM_PAGE: u16 = 0x27; //address must have already been latched with gba dictionary
const GENESIS_ROM_PAGE0: u16 = 0x28; //bank address A17-23 must have been latched already
                                     //TODO come up with better way to handle genesis address complications
const GENESIS_ROM_PAGE1: u16 = 0x29; //bank address A17-23 must have been latched already
const N64_ROM_PAGE: u16 = 0x30;

const NESPPU_1KB_TOGGLE: u16 = 0x31; //similar to PPU page read but /RD signal toggles with each read
const NESCPU_4KB_TOGGLE: u16 = 0x32; //similar to CPU page read but M2 toggles with each read
const GENESIS_RAM_PAGE: u16 = 0x33; //bank address A17-23 must have been latched already

//operand LSB
//SST 39SF0x0 manf/prod IDs
const SST_MANF_ID: u16 = 0xBF;
const SST_PROD_128: u16 = 0xB5;
const SST_PROD_256: u16 = 0xB6;
const SST_PROD_512: u16 = 0xB7;
//SRAM manf/prod ID
const SRAM: u16 = 0xAA;
//MASK ROM read only
const MASKROM: u16 = 0xDD;

//set multiple and add multiple
//miscdata: buffer number
//operMSB: multiple
//operLSB: add multiple
const SET_MULT_N_ADDMULT: u16 = 0x31;

//set mapper and mapper variant
//miscdata: buffer number
//operMSB: mapper
//operLSB: mapper variant
const SET_MAP_N_MAPVAR: u16 = 0x32;
//operand MSB mapper
const NROM: u16 = 0;
const MMC1: u16 = 1;
const UxROM: u16 = 2;
const CNROM: u16 = 3;
const MMC3: u16 = 4;
const MMC5: u16 = 5;
const AxROM: u16 = 7;
const MMC2: u16 = 9;
const MMC4: u16 = 10;
const CDREAM: u16 = 11;
const CNINJA: u16 = 12; //not actually mapper 12, just a temp mapper assignment
const A53: u16 = 28;
const MAP30: u16 = 30;
const EZNSF: u16 = 31;
const BxROM: u16 = 34;
const RAMBO: u16 = 64;
const H3001: u16 = 65; //IREM mapper
const GxROM: u16 = 66;
const SUN3: u16 = 67;
const SUN4: u16 = 68;
const FME7: u16 = 69; //SUNSOFT-5 with synth
const HDIVER: u16 = 78;
const GTROM: u16 = 111;
const DxROM: u16 = 205;

const MM2: u16 = 253;
const DPROM: u16 = 254; //just a random mapper number for whatever I need it for
const MMC3S: u16 = 252;
//	UNKNOWN 255	don't assign to something meaningful
//operand LSB mapper variant
const NOVAR: u16 = 0;

const LOROM: u16 = 0;
const HIROM: u16 = 1; //file starts at bank 40 and is mirrored to C0
const EXHIROM: u16 = 2; //file starts at bank C0
const SOROM: u16 = 3; //12MB star ocean mapping

const LOROM_5VOLT: u16 = 4; //Catskull 5v SNES board with SST PLCC flash
const HIROM_5VOLT: u16 = 5;

const LOROM_3VOLT: u16 = 6;
const HIROM_3VOLT: u16 = 7;

const LOROM_3V_PAGE: u16 = 8;
const HIROM_3V_PAGE: u16 = 9;

const LOROM_3V_VERIFY: u16 = 10; //same as 3VOLT above, but verifies each byte while writing
const HIROM_3V_VERIFY: u16 = 11;

//set function
//miscdata: buffer number
//operMSB: (might be needed if this is a ponter..?)  or might need more than one function def..
//operLSB: function
const SET_FUNCTION: u16 = 0x33;

//const BUFF_OPCODE_NRV_MAX: u16 = 0x3F
//------------------------------------------------------------------------------------------------
//opcodes in this range are PAYLOADS and DO contain buff# in miscdata byte
//const BUFF_PAYLOADN_MIN: u16 = 0x40-4F
//------------------------------------------------------------------------------------------------

//designate what buffer to fill with miscdata byte
//no return value as it's write OUT only
//operandMSB:LSB actually contains first 2 bytes
const BUFF_OUT_PAYLOADN_2B_INSP: u16 = 0x40;

//designate what buffer to fill/read with miscdata byte
const BUFF_PAYLOADN: u16 = 0x41;

//const BUFF_PAYLOADN_MAX: u16 = 0x4F
//------------------------------------------------------------------------------------------------
//opcodes in this range HAVE RETURN besides error code and DO contain buff# in miscdata byte
//const BUFF_OPCODE_RV_MIN: u16 = 0x50-5F
//------------------------------------------------------------------------------------------------

//return buffer elements
//misc/data: buffer number
const GET_PRI_ELEMENTS: u16 = 0x50; //RL=8
                                    //rv0: success/error code
                                    //rv1: rdatalen = 6
                                    //rv2: last_idx
const BUFF_LASTIDX: u16 = 1;
//rv3: status
const BUFF_STATUS: u16 = 2;
//rv4: cur_byte
const BUFF_CURBYTE: u16 = 3;
//rv5: reload
const BUFF_RELOAD: u16 = 4;
//rv6: id
const BUFF_ID: u16 = 5;
//rv7: function
const BUFF_FUNC: u16 = 6;

//return buffer elements
//misc/data: buffer number
const GET_SEC_ELEMENTS: u16 = 0x51; //RL=8
                                    //rv0: success/error code
                                    //rv1: rdatalen = 6
                                    //rv2: mem_type
const BUFF_MEMTYPE: u16 = 1;
//rv3: part_num
const BUFF_PARTNUM: u16 = 2;
//rv4: multiple
const BUFF_MUL: u16 = 3;
//rv5: add_multiple
const BUFF_ADDMUL: u16 = 4;
//rv6: mapper
const BUFF_MAPPER: u16 = 5;
//rv7: mapvar
const BUFF_MAPVAR: u16 = 6;

//return buffer elements
//misc/data: buffer number
const GET_PAGE_NUM: u16 = 0x52; //RL=4
                                //rv0: success/error code
                                //rv1: rdatalen = 2
                                //rv3-2: 16bit page number

//const BUFFN_INMISC_MAX: u16 = 0x5F	//NOTE OVERLAP!!
//------------------------------------------------------------------------------------------------
//opcodes in this range HAVE RETURN value plus error code and DON'T contain buff# in miscdata byte
//			: u16 = 0x60-6F
//------------------------------------------------------------------------------------------------

//send bank number and read back it's status
//0xFF-UNALLOC
//gets assigned buffer ID number when allocated
//operandMSB/miscdata: unused
//operandLSB: raw bank number to retrieve status of
//return value status of that raw bank (set to bank id if allocated)
const GET_RAW_BANK_STATUS: u16 = 0x60; //RL=3
                                       //buffer/operation status values
const EMPTY: u16 = 0x00;
const RESET: u16 = 0x01;
const PROBLEM: u16 = 0x10;
const PREPARING: u16 = 0x20;
const USB_UNLOADING: u16 = 0x80;
const USB_LOADING: u16 = 0x90;
const USB_FULL: u16 = 0x98;
const CHECKING: u16 = 0xC0;
const DUMPING: u16 = 0xD0;
const STARTDUMP: u16 = 0xD2;
const DUMPED: u16 = 0xD8;
const ERASING: u16 = 0xE0;
const FLASHING: u16 = 0xF0;
const STARTFLASH: u16 = 0xF2;
const FLASHED: u16 = 0xF4;
const FLASH_WAIT: u16 = 0xF8;
const STOPPED: u16 = 0xFE;
const UNALLOC: u16 = 0xFF;

//retrieve cur_buff status
const GET_CUR_BUFF_STATUS: u16 = 0x61; //RL=3

//const BUFF_OPCODE_RV_MAX: u16 = 0x6F
//------------------------------------------------------------------------------------------------
//opcodes in this range are PAYLOADS and DO NOT contain buff# in miscdata byte
//const BUFF_PAYLOAD_MIN: u16 = 0x70-7F
//------------------------------------------------------------------------------------------------

//does NOT designate what buffer to fill with opcode
//endpoint direction determines if read/write
//no operands no return value aside from payload for transfer IN
//max size for these transfers is 254Bytes for IN and OUT
const BUFF_PAYLOAD: u16 = 0x70;

//does NOT designate what buffer to fill with opcode
//no return value as it's write OUT only
//operandMSB:LSB actually contains first 2 bytes
const BUFF_OUT_PAYLOAD_2B_INSP: u16 = 0x71;

//const BUFF_PAYLOAD_MAX: u16 = 0x7F

//=============================================================================================
//	OPCODES with up to 24bit operand and no return value besides SUCCESS/ERROR_CODE
//	BUFFER NUMBER denoted in lower nibble of opcode
//=============================================================================================
//	Detect this opcode group which uses 3 LSbits to determine which buffer to call
const BUFF_OPCODE_BUFN_MIN: u16 = 0x80;
const BUFF_OPCODE_BUFN_MAX: u16 = 0xFF;
//
//
//	Detect this opcode/operand setup with opcode between the following defines:
const BUFF_OPCODE_BUFN_NRV_MIN: u16 = 0x80;
const BUFF_OPCODE_BUFN_NRV_MAX: u16 = 0xBF;
//
const BUFF_OPCODE_BUFN_RV_MIN: u16 = 0xC0;
const BUFF_OPCODE_BUFN_RV_MAX: u16 = 0xEF;
//
const BUFF_OPCODE_PAYLOAD_MIN: u16 = 0xF0;
const BUFF_OPCODE_PAYLOAD_MAX: u16 = 0xFF;
//=============================================================================================
//=============================================================================================

//allocate firmware sram to a buffer
//send a buffer number
//buffer size
//base address 0-255 (in 32byte chunks)
//returns SUCCESS if able to allocate
//returns error code if unable to allocate
//operMSB: id to give to new buffer
//	(upper id bits used to set any address bits not covered by page and buff size if needed)
//operLSB: base bank number
//misc/data: size (number of banks to allocate to buffer)
//	-size doesn't get stored in buffer, the last_idx does
const ALLOCATE_BUFFER0: u16 = 0x80;
const ALLOCATE_BUFFER1: u16 = 0x81;
const ALLOCATE_BUFFER2: u16 = 0x82;
const ALLOCATE_BUFFER3: u16 = 0x83;
const ALLOCATE_BUFFER4: u16 = 0x84;
const ALLOCATE_BUFFER5: u16 = 0x85;
const ALLOCATE_BUFFER6: u16 = 0x86;
const ALLOCATE_BUFFER7: u16 = 0x87;

//SET BUFFER ELEMENTS

//set reload and page_num
//misc/data reload
//operMSB:LSB page_num (16 bit)
const SET_RELOAD_PAGENUM0: u16 = 0x90;
const SET_RELOAD_PAGENUM1: u16 = 0x91;
const SET_RELOAD_PAGENUM2: u16 = 0x92;
const SET_RELOAD_PAGENUM3: u16 = 0x93;
const SET_RELOAD_PAGENUM4: u16 = 0x94;
const SET_RELOAD_PAGENUM5: u16 = 0x95;
const SET_RELOAD_PAGENUM6: u16 = 0x96;
const SET_RELOAD_PAGENUM7: u16 = 0x97;

//designate what buffer to fill with opcode
//endpoint direction determines if read/write
//no operands no return value aside from payload for transfer IN
//DOES NOT STUFF extra bytes in setup packet for write/OUT transfers
const BUFF_PAYLOAD0: u16 = 0xF0;
const BUFF_PAYLOAD1: u16 = 0xF1;
const BUFF_PAYLOAD2: u16 = 0xF2;
const BUFF_PAYLOAD3: u16 = 0xF3;
const BUFF_PAYLOAD4: u16 = 0xF4;
const BUFF_PAYLOAD5: u16 = 0xF5;
const BUFF_PAYLOAD6: u16 = 0xF6;
const BUFF_PAYLOAD7: u16 = 0xF7;

pub fn raw_buffer_reset<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    util::read_device(device_handle, &mut buf, request, RAW_BUFFER_RESET, 0, 0);
}

pub fn allocate_buffer0<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    let value: u16 = (misc << 8) | ALLOCATE_BUFFER0;
    util::read_device(device_handle, &mut buf, request, value, operand, misc);
}

pub fn allocate_buffer1<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    let value: u16 = (misc << 8) | ALLOCATE_BUFFER1;
    util::read_device(device_handle, &mut buf, request, value, operand, misc);
}

pub fn set_reload_pagenum0<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    operand: u16,
    misc: u16,
) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    let value: u16 = (misc << 8) | SET_RELOAD_PAGENUM0;
    util::read_device(device_handle, &mut buf, request, value, operand, misc);
}

pub fn set_reload_pagenum1<T: UsbContext>(
    device_handle: &DeviceHandle<T>,
    operand: u16,
    misc: u16,
) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    let value: u16 = (misc << 8) | SET_RELOAD_PAGENUM1;
    util::read_device(device_handle, &mut buf, request, value, operand, misc);
}

pub fn set_map_n_mapvar<T: UsbContext>(device_handle: &DeviceHandle<T>, operand: u16, misc: u16) {
    let request = 5; // 5 is for buffer
    let mut buf: [u8; 1] = [0; 1];
    let value: u16 = (misc << 8) | SET_MAP_N_MAPVAR;
    util::read_device(device_handle, &mut buf, request, value, operand, misc);
}
