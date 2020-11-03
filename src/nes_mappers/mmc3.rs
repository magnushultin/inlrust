use rusb::{DeviceHandle, UsbContext};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

use crate::io;
use crate::nes::{detect_mapper_mirroring, ppu_wr, ppu_rd, cpu_rd, cpu_wr, dump, Mirroring, ppu_ram_sense};
use crate::opcodes::buffer as op_buffer;

pub fn test_mmc3<T: UsbContext>(device_handle: &DeviceHandle<T>) {
    println!("Testing MMC3");
    //mirror_test
    init_mapper(&device_handle);

    cpu_wr(&device_handle, 0xA000, 0x00);
    if detect_mapper_mirroring(&device_handle).unwrap() != Mirroring::VERT {
        println!("MMC3 mirror test fail (Vertical)");
    }

    cpu_wr(&device_handle, 0xA000, 0x01);
    if detect_mapper_mirroring(&device_handle).unwrap() != Mirroring::HORZ {
        println!("MMC3 mirror test fail (Horizontal)");
    }

    ppu_ram_sense(&device_handle, 0x1000);
    println!("EXP0 pull-up test: {}", io::exp0_pullup_test(&device_handle));

    // prgrom manf id
    // Same as mmc1 except init
    init_mapper(&device_handle);
    cpu_wr(&device_handle, 0xD555, 0xAA);
    cpu_wr(&device_handle, 0xAAAA, 0x55);
    cpu_wr(&device_handle, 0xD555, 0x90);

    let rv = cpu_rd(&device_handle, 0x8000);
    println!("PRG-ROM manf ID: 0x{:x}", rv);

    let rv = cpu_rd(&device_handle, 0x8001);
    println!("PRG-ROM prod ID: 0x{:x}", rv);

    // Exit
    cpu_wr(&device_handle, 0x8000, 0xF0);

    //    read CHR-ROM manf ID
    init_mapper(&device_handle);
    ppu_wr(&device_handle, 0x1555, 0xAA);
    ppu_wr(&device_handle, 0x1AAA, 0x55);
    ppu_wr(&device_handle, 0x1555, 0x90);

    let rv = ppu_rd(&device_handle, 0x0000);
    println!("CHR-ROM manf ID: 0x{:x}", rv);

    let rv = ppu_rd(&device_handle, 0x0001);
    println!("CHR-ROM prod ID: 0x{:x}", rv);
    // EXIT
    ppu_wr(&device_handle, 0x0000, 0xF0);
}

pub fn init_mapper<T: UsbContext>(device_handle: &DeviceHandle<T>) {

    // for save data safety start by disabling WRAM, and deny writes
    cpu_wr(&device_handle, 0xA001, 0x40);

    // set mirroring
    cpu_wr(&device_handle, 0xA000, 0x00);
	
    // $8000-9FFE even
    // MMC3 bank select:
    // 7  bit  0
    // ---- ----
    // CPMx xRRR
    // |||   |||
    // |||   +++- Specify which bank register to update on next write to Bank Data register
    // |||        0: Select 2 KB CHR bank at PPU $0000-$07FF (or $1000-$17FF);
    // |||        1: Select 2 KB CHR bank at PPU $0800-$0FFF (or $1800-$1FFF);
    // |||        2: Select 1 KB CHR bank at PPU $1000-$13FF (or $0000-$03FF);
    // |||        3: Select 1 KB CHR bank at PPU $1400-$17FF (or $0400-$07FF);
    // |||        4: Select 1 KB CHR bank at PPU $1800-$1BFF (or $0800-$0BFF);
    // |||        5: Select 1 KB CHR bank at PPU $1C00-$1FFF (or $0C00-$0FFF);
    // |||        6: Select 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF);
    // |||        7: Select 8 KB PRG ROM bank at $A000-$BFFF
    // ||+------- Nothing on the MMC3, see MMC6
    // |+-------- PRG ROM bank mode (0: $8000-$9FFF swappable,
    // |                                $C000-$DFFF fixed to second-last bank;
    // |                             1: $C000-$DFFF swappable,
    // |                                $8000-$9FFF fixed to second-last bank)
    // +--------- CHR A12 inversion (0: two 2 KB banks at $0000-$0FFF,
    //                                  four 1 KB banks at $1000-$1FFF;
    //                               1: two 2 KB banks at $1000-$1FFF, 
    //                                  four 1 KB banks at $0000-$0FFF)

    // For CHR-ROM flash writes, use lower 4KB (PT0) for writting data & upper 4KB (PT1) for commands
    cpu_wr(&device_handle, 0x8000, 0x00);
    cpu_wr(&device_handle, 0x8001, 0x00); // 2KB @ PPU $0000

    cpu_wr(&device_handle, 0x8000, 0x01);
    cpu_wr(&device_handle, 0x8001, 0x02); // 2KB @ PPU $0800

    // use lower half of PT1 for $5555 commands
    cpu_wr(&device_handle, 0x8000, 0x02);
    cpu_wr(&device_handle, 0x8001, 0x15); // 1KB @ PPU $1000

    cpu_wr(&device_handle, 0x8000, 0x03);
    cpu_wr(&device_handle, 0x8001, 0x15); // 1KB @ PPU $1400
    
    // use upper half of PT1 for $2AAA commands
    cpu_wr(&device_handle, 0x8000, 0x04);
    cpu_wr(&device_handle, 0x8001, 0x0A); // 1KB @ PPU $1800

    cpu_wr(&device_handle, 0x8000, 0x05);
    cpu_wr(&device_handle, 0x8001, 0x0A); // 1KB @ PPU $1C00

    // For PRG-ROM flash writes:
    // mode 0: $C000-FFFF fixed to last 16KByte
    //         reg6 controls $8000-9FFF ($C000-DFFF in mode 1)
    //         reg7 controls $A000-BFFF (regardless of mode)
    // Don't want to write data to $8000-9FFF because those are the bank regs
    // Writting data to $A000-BFFF is okay as that will only affect mirroring and WRAM ctl
    // 
    // $5555 commands can be written to $D555 (A14 set, A13 clear)
    // $2AAA commands must be written through reg6/7 ($8000-BFFF) to clear A14 & set A13
    // 	reg7 ($A000-BFFF) is ideal because it won't affect banking, just mirror/WRAM
    // 	actually $2AAA is even, so it'll only affect mirroring which is ideal
    // DATA writes can occur at $8000-9FFF, but care must be taken to maintain banking.
    // 	Setting $8000 to a CHR bank prevents DATA writes from changing PRG banks
    // 	The DATA write will change the bank select if it's written to an even address though
    // 	To cover this, simply select the CHR bank again with $8000 reg after the data write
    // 	Those DATA writes can also corrupt the PRG/CHR modes, so just always follow
    // 	DATA writes by writting 0x00 to $8000

    // $5555 commands written to $D555 (default due to mode 0)
    // $2AAA commands written to $AAAA

    cpu_wr(&device_handle, 0x8000, 0x07);
    cpu_wr(&device_handle, 0x8001, 0x01); // 8KB @ CPU $A000

    // DATA writes written to $8000-9FFF
    cpu_wr(&device_handle, 0x8000, 0x06);
    cpu_wr(&device_handle, 0x8001, 0x00); // 8KB @ CPU $8000

    // set $8000 bank select register to a CHR reg so $8000/1 writes don't change the PRG bank
    cpu_wr(&device_handle, 0x8000, 0x00);
}

pub fn dump_prgrom<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u16,
) {
    let mut kb_per_read = 16;
    let num_reads = rom_size_kb / kb_per_read;
    let mut read_count = 0;
    let addr_base = 0x08;

    while read_count < num_reads {
        cpu_wr(&device_handle, 0x8000, 0x06);
        cpu_wr(&device_handle, 0x8001, read_count * 2); // 8KB @ CPU $8000

        cpu_wr(&device_handle, 0x8000, 0x07);
        cpu_wr(&device_handle, 0x8001, read_count*2 + 1); // 8KB @ CPU $A000

        dump(&device_handle, file, kb_per_read, addr_base, op_buffer::NESCPU_4KB);
        read_count += 1;
    }
}

pub fn dump_chrrom<T: UsbContext, W: Write>(
    device_handle: &DeviceHandle<T>,
    file: &mut BufWriter<W>,
    rom_size_kb: u16,
) {
    let kb_per_read = 4;
    let num_reads = rom_size_kb / kb_per_read;
    let mut read_count = 0;
    let addr_base = 0x00;

    while read_count < num_reads {

        cpu_wr(&device_handle, 0x8000, 0x00);
        cpu_wr(&device_handle, 0x8001, (read_count*2)<<1); // 2KB @ PPU $0000

        cpu_wr(&device_handle, 0x8000, 0x01);
        cpu_wr(&device_handle, 0x8001, (read_count*2 + 1)<<1); // 2KB @ PPU $0800

        dump(&device_handle, file, kb_per_read, addr_base, op_buffer::NESPPU_1KB);
        read_count += 1;
    }
}
