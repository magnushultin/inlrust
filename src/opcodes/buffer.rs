pub const RAW_BUFFER_RESET: u16 = 0x00;
pub const SET_MEM_N_PART: u16 = 0x30;
pub const NESCPU_4KB: u16 = 0x20;
pub const NESPPU_1KB: u16 = 0x21;
pub const SNESROM_PAGE: u16 = 0x24; 
pub const SNESSYS_PAGE: u16 = 0x25;
pub const GAMEBOY_PAGE: u16 = 0x26; 
pub const GBA_ROM_PAGE: u16 = 0x27; 
pub const GENESIS_ROM_PAGE0: u16 = 0x28; 
pub const GENESIS_ROM_PAGE1: u16 = 0x29; 
pub const GENESIS_RAM_PAGE: u16 = 0x33;

pub const SET_MAP_N_MAPVAR: u16 = 0x32;
pub const GET_CUR_BUFF_STATUS: u16 = 0x61; //RL=3
pub const BUFF_PAYLOAD: u16 = 0x70;

pub const ALLOCATE_BUFFER0: u16 = 0x80;
pub const ALLOCATE_BUFFER1: u16 = 0x81;

pub const SET_RELOAD_PAGENUM0: u16 = 0x90;
pub const SET_RELOAD_PAGENUM1: u16 = 0x91;