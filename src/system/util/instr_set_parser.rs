use std::collections::HashMap;
use std::fs;
use std::usize;

use crate::system::cpu::{Cpu, Op, OpInfo, AddrMode, Instruction};

pub struct InstrSetParser {
    pub addr_mode_map: HashMap<&'static str, (AddrMode, fn (cpu: &mut Cpu))>,
    pub instr_map: HashMap<&'static str, (Instruction, fn (cpu: &mut Cpu))>,
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
            instruction:  Cpu::nop,
        };

        InstrSetParser {
            addr_mode_map: HashMap::from([
                ("acc",     (AddrMode::Acc, Cpu::acc as fn (&mut Cpu))),
                ("imm",     (AddrMode::Imm, Cpu::imm)),
                ("abs",     (AddrMode::Abs, Cpu::abs)),
                ("zp",      (AddrMode::Zp, Cpu::zp)),
                ("zpx",     (AddrMode::Zpx, Cpu::zpx)),
                ("zpy",     (AddrMode::Zpy, Cpu::zpy)),
                ("absx",    (AddrMode::Absx ,Cpu::absx)),
                ("absy",    (AddrMode::Absy ,Cpu::absy)),
                ("imp",     (AddrMode::Imp ,Cpu::imp)),
                ("rel",     (AddrMode::Rel ,Cpu::rel)),
                ("indx",    (AddrMode::Indx ,Cpu::indx)),
                ("indy",    (AddrMode::Indy ,Cpu::indy)),
                ("ind",     (AddrMode::Ind ,Cpu::ind)),
            ]),
            instr_map: HashMap::from([
                ("adc", (Instruction::Adc, Cpu::adc as fn (&mut Cpu))),
                ("and", (Instruction::And, Cpu::and)),
                ("asl", (Instruction::Asl, Cpu::asl)),
                ("bcc", (Instruction::Bcc, Cpu::bcc)),
                ("bcs", (Instruction::Bcs, Cpu::bcs)),
                ("beq", (Instruction::Beq, Cpu::beq)),
                ("bit", (Instruction::Bit, Cpu::bit)),
                ("bmi", (Instruction::Bmi, Cpu::bmi)),
                ("bne", (Instruction::Bne, Cpu::bne)),
                ("bpl", (Instruction::Bpl, Cpu::bpl)),
                ("brk", (Instruction::Brk, Cpu::brk)),
                ("bvc", (Instruction::Bvc, Cpu::bvc)),
                ("bvs", (Instruction::Bvs, Cpu::bvs)),
                ("clc", (Instruction::Clc, Cpu::clc)),
                ("cld", (Instruction::Cld, Cpu::cld)),
                ("cli", (Instruction::Cli, Cpu::cli)),
                ("clv", (Instruction::Clv, Cpu::clv)),
                ("cmp", (Instruction::Cmp, Cpu::cmp)),
                ("cpx", (Instruction::Cpx, Cpu::cpx)),
                ("cpy", (Instruction::Cpy, Cpu::cpy)),
                ("dec", (Instruction::Dec, Cpu::dec)),
                ("dex", (Instruction::Dex, Cpu::dex)),
                ("dey", (Instruction::Dey, Cpu::dey)),
                ("eor", (Instruction::Eor, Cpu::eor)),
                ("inc", (Instruction::Inc, Cpu::inc)),
                ("inx", (Instruction::Inx, Cpu::inx)),
                ("iny", (Instruction::Iny, Cpu::iny)),
                ("jmp", (Instruction::Jmp, Cpu::jmp)),
                ("jsr", (Instruction::Jsr, Cpu::jsr)),
                ("lda", (Instruction::Lda, Cpu::lda)),
                ("ldx", (Instruction::Ldx, Cpu::ldx)),
                ("ldy", (Instruction::Ldy, Cpu::ldy)),
                ("lsr", (Instruction::Lsr, Cpu::lsr)),
                ("nop", (Instruction::Nop, Cpu::nop)),
                ("ora", (Instruction::Ora, Cpu::ora)),
                ("pha", (Instruction::Pha, Cpu::pha)),
                ("php", (Instruction::Php, Cpu::php)),
                ("pla", (Instruction::Pla, Cpu::pla)),
                ("plp", (Instruction::Plp, Cpu::plp)),
                ("rol", (Instruction::Rol, Cpu::rol)),
                ("ror", (Instruction::Ror, Cpu::ror)),
                ("rti", (Instruction::Rti, Cpu::rti)),
                ("rts", (Instruction::Rts, Cpu::rts)),
                ("sbc", (Instruction::Sbc, Cpu::sbc)),
                ("sec", (Instruction::Sec, Cpu::sec)),
                ("sed", (Instruction::Sed, Cpu::sed)),
                ("sei", (Instruction::Sei, Cpu::sei)),
                ("sta", (Instruction::Sta, Cpu::sta)),
                ("stx", (Instruction::Stx, Cpu::stx)),
                ("sty", (Instruction::Sty, Cpu::sty)),
                ("tax", (Instruction::Tax, Cpu::tax)),
                ("tay", (Instruction::Tay, Cpu::tay)),
                ("tsx", (Instruction::Tsx, Cpu::tsx)),
                ("txa", (Instruction::Txa, Cpu::txa)),
                ("txs", (Instruction::Txs, Cpu::txs)),
                ("tya", (Instruction::Tya, Cpu::tya)),
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
            },
        };

        match usize::from_str_radix(value_stripped, 16) {
            Ok(n) => Ok(n),
            Err(_) => Err(format!("Invalid opcode: {}", value)),
        }
    }


    fn parse_line(&self, line: &str) -> Result<(usize, Op), String> {
        let tokens: Vec<_> = line.split_whitespace().collect();
        match tokens.as_slice() {
            [opcode, instr, addr_mode] => {
                println!("opcode: {}, instr: {}, addr_mode: {}",
                         opcode, instr, addr_mode);

                let opcode = self.parse_opcode(opcode)?;

                let (instr, instr_ptr) = match self.instr_map.get(instr) {
                    Some(value) => value.clone(),
                    None => {
                        return Err(format!("Invalid instruction: {}", instr));
                    },
                };

                let (addr_mode, addr_mode_ptr) = match self.addr_mode_map.get(addr_mode) {
                    Some(value) => value.clone(),
                    None => {
                        return Err(format!("Invalid addressing mode: {}", addr_mode));
                    },
                };

                Ok((opcode, Op {
                    info: OpInfo {
                        instruction: instr,
                        address_mode: addr_mode,
                    },
                    instruction: instr_ptr,
                    address_mode: addr_mode_ptr,}))
            }

            _ => Err(format!("Invalid line: {}", line))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Op>, String> {
        let nop = Op {
            address_mode: Cpu::imp,
            instruction:  Cpu::nop,
            info: OpInfo {
                address_mode: AddrMode::Imp,
                instruction: Instruction::Nop,
            }
        };

        let mut optable: Vec<Op> = vec![nop; 0x100];

        let file = fs::read_to_string(self.filepath.as_str())
            .expect("Error reading instruction set file");
        for line in file.lines() {
            if line.is_empty() {
                continue;
            }
            println!("parsing: {}", line);

            let (opcode, op) = self.parse_line(line).unwrap();
            optable[opcode] = op;
        }

        Ok(optable)
    }
}
