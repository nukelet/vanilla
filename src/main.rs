mod mmio;
mod system;

use std::fs;

use system::cpu::{Cpu, Flag, bit};

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
    let mut cpu =Cpu::new();
    cpu.regs.a = 0x80;
    cpu.mmio.write(0x0000, &[0x69, 0xFF, 0x69, 0x80]);
    let mem = cpu.mmio.read(0x0000, 32);
    print_memory(&mem, 0x0000);
    cpu.step();
    dump_regs(&cpu);

    println!("{}", bit!(0b01000000, 6));

    let a = 0x10F;
    println!("{}", a as u8);

    let mut parser = InstrSetParser::new("instructions.txt");
    let optable = parser.parse()
        .expect("Parsing error");

    let data = vec![0x69, 0xFF, 0x69, 0x80];
    let data = fs::read("6502_functional_test.bin").unwrap();
    let mut disassembler = Disassembler::new(data, &optable);
    disassembler.read(0x400);
}