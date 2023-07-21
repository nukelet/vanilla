use crate::system::cpu::{AddrMode, Op};

pub struct Disassembler<'a> {
    pub data: Vec<u8>,
    pub optable: &'a Vec<Op>,
}

fn get_arg_count(op: &Op) -> u32 {
    match op.info.address_mode {
        AddrMode::Imp | AddrMode::Acc => 0,

        AddrMode::Zp
        | AddrMode::Imm
        | AddrMode::Indx
        | AddrMode::Indy
        | AddrMode::Zpx
        | AddrMode::Zpy
        | AddrMode::Rel => 1,

        AddrMode::Ind | AddrMode::Abs | AddrMode::Absx | AddrMode::Absy => 2,
    }
}

fn get_op_fmt_string(op: &Op, arg: u16) -> String {
    let instr_string: String = format!("{:?}", op.info.instruction).to_uppercase();
    let arg_string: String = match op.info.address_mode {
        AddrMode::Imp => format!(""),
        AddrMode::Acc => format!(" A"),
        AddrMode::Imm => format!(" #{:#04x}", arg),
        AddrMode::Rel => format!(" ${:#04x}", arg),
        AddrMode::Zp => format!(" ${:#04x}", arg),
        AddrMode::Zpx => format!(" ${:#04x}", arg),
        AddrMode::Zpy => format!(" ${:#04x}", arg),
        AddrMode::Ind => format!(" (${:#06x})", arg),
        AddrMode::Abs => format!(" ${:#06x}", arg),
        AddrMode::Absx => format!(" ${:#06x}, X", arg),
        AddrMode::Absy => format!(" ${:#06x}, Y", arg),
        AddrMode::Indx => format!(" (${:#04x}, X)", arg),
        AddrMode::Indy => format!(" (${:#04x}), Y", arg),
    };

    format!("{} {}", instr_string, arg_string)
}

impl<'a> Disassembler<'a> {
    pub fn new(data: Vec<u8>, optable: &'a Vec<Op>) -> Disassembler {
        Disassembler { data, optable }
    }

    pub fn read(&mut self, offset: usize) {
        let mut it = self.data.iter().enumerate();
        while let Some((pc, &byte)) = it.next() {
            // TODO: maybe do this more elegantly?
            if pc < offset {
                continue;
            }

            let op = self.optable.get(byte as usize).unwrap();
            let opcode = byte;
            let arg_count = get_arg_count(op);
            let arg: u16 = match arg_count {
                0 => 0,
                1 => {
                    let (_, &arg) = it.next().expect("Unexpected EOF when parsing");
                    arg as u16
                }

                2 => {
                    let (_, &arg_lo) = it.next().expect("Unexpected EOF when parsing");

                    let (_, &arg_hi) = it.next().expect("Unexpected EOF when parsing");

                    ((arg_hi as u16) << 8) + (arg_lo as u16)
                }

                _ => {
                    panic!("Parsing error: invalid arg_count");
                }
            };

            let line = get_op_fmt_string(op, arg);
            println!("{:#06X}: {}", pc, line);
        }
    }
}
