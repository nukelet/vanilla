use std::collections::HashMap;
use std::fs;
use std::usize;

use crate::system::cpu::{AddrMode, Cpu, Instruction, Op, OpInfo};

pub struct InstrSetParser {
    pub addr_mode_map: HashMap<&'static str, (AddrMode, fn(cpu: &mut Cpu))>,
    pub instr_map: HashMap<&'static str, (Instruction, fn(cpu: &mut Cpu))>,
    pub optable: Vec<Op>,
    pub filepath: String,
}

impl InstrSetParser {
    pub fn new(filepath: &str) -> InstrSetParser {
        let nop = Op {
            info: OpInfo {
                address_mode: AddrMode::Imp,
                instruction: Instruction::Nop,
            },

            address_mode: Cpu::imp,
            instruction: Cpu::nop,
            cycles: 1,
        };

        InstrSetParser {
            addr_mode_map: HashMap::from([
                ("ACC", (AddrMode::Acc, Cpu::acc as fn(&mut Cpu))),
                ("IMM", (AddrMode::Imm, Cpu::imm)),
                ("ABS", (AddrMode::Abs, Cpu::abs)),
                ("ZP", (AddrMode::Zp, Cpu::zp)),
                ("ZPX", (AddrMode::Zpx, Cpu::zpx)),
                ("ZPY", (AddrMode::Zpy, Cpu::zpy)),
                ("ABSX", (AddrMode::Absx, Cpu::absx)),
                ("ABSY", (AddrMode::Absy, Cpu::absy)),
                ("IMP", (AddrMode::Imp, Cpu::imp)),
                ("REL", (AddrMode::Rel, Cpu::rel)),
                ("INDX", (AddrMode::Indx, Cpu::indx)),
                ("INDY", (AddrMode::Indy, Cpu::indy)),
                ("IND", (AddrMode::Ind, Cpu::ind)),
            ]),
            instr_map: HashMap::from([
                ("ADC", (Instruction::Adc, Cpu::adc as fn(&mut Cpu))),
                ("AND", (Instruction::And, Cpu::and)),
                ("ASL", (Instruction::Asl, Cpu::asl)),
                ("BCC", (Instruction::Bcc, Cpu::bcc)),
                ("BCS", (Instruction::Bcs, Cpu::bcs)),
                ("BEQ", (Instruction::Beq, Cpu::beq)),
                ("BIT", (Instruction::Bit, Cpu::bit)),
                ("BMI", (Instruction::Bmi, Cpu::bmi)),
                ("BNE", (Instruction::Bne, Cpu::bne)),
                ("BPL", (Instruction::Bpl, Cpu::bpl)),
                ("BRK", (Instruction::Brk, Cpu::brk)),
                ("BVC", (Instruction::Bvc, Cpu::bvc)),
                ("BVS", (Instruction::Bvs, Cpu::bvs)),
                ("CLC", (Instruction::Clc, Cpu::clc)),
                ("CLD", (Instruction::Cld, Cpu::cld)),
                ("CLI", (Instruction::Cli, Cpu::cli)),
                ("CLV", (Instruction::Clv, Cpu::clv)),
                ("CMP", (Instruction::Cmp, Cpu::cmp)),
                ("CPX", (Instruction::Cpx, Cpu::cpx)),
                ("CPY", (Instruction::Cpy, Cpu::cpy)),
                ("DEC", (Instruction::Dec, Cpu::dec)),
                ("DEX", (Instruction::Dex, Cpu::dex)),
                ("DEY", (Instruction::Dey, Cpu::dey)),
                ("EOR", (Instruction::Eor, Cpu::eor)),
                ("INC", (Instruction::Inc, Cpu::inc)),
                ("INX", (Instruction::Inx, Cpu::inx)),
                ("INY", (Instruction::Iny, Cpu::iny)),
                ("JMP", (Instruction::Jmp, Cpu::jmp)),
                ("JSR", (Instruction::Jsr, Cpu::jsr)),
                ("LDA", (Instruction::Lda, Cpu::lda)),
                ("LDX", (Instruction::Ldx, Cpu::ldx)),
                ("LDY", (Instruction::Ldy, Cpu::ldy)),
                ("LSR", (Instruction::Lsr, Cpu::lsr)),
                ("NOP", (Instruction::Nop, Cpu::nop)),
                ("ORA", (Instruction::Ora, Cpu::ora)),
                ("PHA", (Instruction::Pha, Cpu::pha)),
                ("PHP", (Instruction::Php, Cpu::php)),
                ("PLA", (Instruction::Pla, Cpu::pla)),
                ("PLP", (Instruction::Plp, Cpu::plp)),
                ("ROL", (Instruction::Rol, Cpu::rol)),
                ("ROR", (Instruction::Ror, Cpu::ror)),
                ("RTI", (Instruction::Rti, Cpu::rti)),
                ("RTS", (Instruction::Rts, Cpu::rts)),
                ("SBC", (Instruction::Sbc, Cpu::sbc)),
                ("SEC", (Instruction::Sec, Cpu::sec)),
                ("SED", (Instruction::Sed, Cpu::sed)),
                ("SEI", (Instruction::Sei, Cpu::sei)),
                ("STA", (Instruction::Sta, Cpu::sta)),
                ("STX", (Instruction::Stx, Cpu::stx)),
                ("STY", (Instruction::Sty, Cpu::sty)),
                ("TAX", (Instruction::Tax, Cpu::tax)),
                ("TAY", (Instruction::Tay, Cpu::tay)),
                ("TSX", (Instruction::Tsx, Cpu::tsx)),
                ("TXA", (Instruction::Txa, Cpu::txa)),
                ("TXS", (Instruction::Txs, Cpu::txs)),
                ("TYA", (Instruction::Tya, Cpu::tya)),
            ]),
            optable: vec![nop; 0x100],
            filepath: String::from(filepath),
        }
    }

    fn parse_opcode(&self, value: &str) -> Result<usize, String> {
        let value_stripped = match value.strip_prefix("0x") {
            Some(v) => v,
            None => {
                return Err(format!("Invalid opcode: {}", value));
            }
        };

        match usize::from_str_radix(value_stripped, 16) {
            Ok(n) => Ok(n),
            Err(_) => Err(format!("Invalid opcode: {}", value)),
        }
    }

    fn parse_line(&self, line: &str) -> Result<(usize, Op), String> {
        let tokens: Vec<_> = line.split(",").collect();
        match tokens.as_slice() {
            [opcode, instr, addr_mode, size, cycles, flags] => {
                // println!("opcode: {}, instr: {}, addr_mode: {}",
                //          opcode, instr, addr_mode);

                let opcode = self.parse_opcode(opcode)?;
                let cycles: u8 = cycles.parse().unwrap();

                let (instr, instr_ptr) = match self.instr_map.get(instr) {
                    Some(value) => value.clone(),
                    None => {
                        return Err(format!("Invalid instruction: {}", instr));
                    }
                };

                let (addr_mode, addr_mode_ptr) = match self.addr_mode_map.get(addr_mode) {
                    Some(value) => value.clone(),
                    None => {
                        return Err(format!("Invalid addressing mode: {}", addr_mode));
                    }
                };

                Ok((
                    opcode,
                    Op {
                        info: OpInfo {
                            instruction: instr,
                            address_mode: addr_mode,
                        },
                        instruction: instr_ptr,
                        address_mode: addr_mode_ptr,
                        cycles: cycles,
                    },
                ))
            }

            _ => Err(format!("Invalid line: {}", line)),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Op>, String> {
        let nop = Op {
            address_mode: Cpu::imp,
            instruction: Cpu::nop,
            cycles: 1,
            info: OpInfo {
                address_mode: AddrMode::Imp,
                instruction: Instruction::Nop,
            },
        };

        let mut optable: Vec<Op> = vec![nop; 0x100];

        let file =
            fs::read_to_string(self.filepath.as_str()).expect("Error reading instruction set file");
        for line in file.lines() {
            if line.is_empty() {
                continue;
            }
            // println!("parsing: {}", line);

            match self.parse_line(line) {
                Ok((opcode, op)) => {
                    optable[opcode] = op;
                }
                Err(_) => {
                    continue;
                }
            }
            let (opcode, op) = self.parse_line(line).unwrap();
            optable[opcode] = op;
        }

        Ok(optable)
    }
}
