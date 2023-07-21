#![allow(dead_code)]

mod mmio;
mod system;

use std::fs;

use system::cpu::{bit, Cpu, Flag};

fn print_memory(bytes: &[u8], start_addr: u16) {
    for (i, byte) in bytes.iter().enumerate() {
        println!("{:#06x}: {:#04x}", start_addr as usize + i, byte);
    }
}

fn dump_regs(cpu: &Cpu) {
    let regs = &cpu.regs;
    println!(
        "====[REGS]====\n\
         A:  {:#04x}\n\
         X:  {:#04x}\n\
         Y:  {:#04x}\n\
         S:  {:#04x}\n\
         P:  {:#04x}\n\
         PC: {:#06x}\n\
         ====[FLAGS]====\n\
         N: {}\n\
         V: {}\n\
         D: {}\n\
         I: {}\n\
         Z: {}\n\
         C: {}\n\
         ",
        regs.a,
        regs.x,
        regs.y,
        regs.s,
        regs.p,
        regs.pc,
        cpu.get_flag(Flag::N),
        cpu.get_flag(Flag::V),
        cpu.get_flag(Flag::D),
        cpu.get_flag(Flag::I),
        cpu.get_flag(Flag::Z),
        cpu.get_flag(Flag::C),
    );
}

use system::util::disassembler::Disassembler;
use system::util::instr_set_parser::InstrSetParser;

fn main() {
    let mut cpu = Cpu::new();
    cpu.regs.a = 0x50;
    cpu.operand = 0xB0;
    cpu.sbc();
    dump_regs(&cpu);

    let a: u16 = 0x20;
    let b: i8 = -1;
    println!("0xFF extends to {:#4X}", b as u16);
    println!("{:#4X}", a.wrapping_add(b as u16));

    // let mut parser = InstrSetParser::new("resources/6502ops.csv");
    // let optable = parser.parse()
    //     .expect("Parsing error");

    // let a: u8 = 0xFF;
    // println!("{}", a.wrapping_add(1));

    // let data = fs::read("resources/6502_functional_test.bin").unwrap();
    // let mut disassembler = Disassembler::new(data, &optable);
    // disassembler.read(0x400);
}
