#![allow(dead_code)] // TODO: remove

pub struct Regs {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p: u8,
    pub pc: u16,
}

// NVssDIZC
pub enum Flag {
    N = 7, // Negative
    V = 6, // Overflow
    D = 3, // Decimal
    I = 2, // Interrupt disable
    Z = 1, // Zero
    C = 0, // Carry
}

impl Default for Regs {
    fn default() -> Regs {
        Regs {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            s: 0x00,
            p: 0x00,
            pc: 0x0000,
        }
    }
}

pub struct Status {
    offset: i8,
    addr: u16,
    operand: u8,
    cycles: u8,
    page_crossed: bool,
}

impl Default for Status {
    fn default() -> Status {
        Status {
            offset: 0x0,
            addr: 0x0,
            operand: 0x0,
            cycles: 0,
            page_crossed: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AddrMode {
    Acc,
    Imm,
    Abs,
    Zp,
    Zpx,
    Zpy,
    Absx,
    Absy,
    Imp,
    Rel,
    Indx,
    Indy,
    Ind,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Instruction {
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
    Sec,
    Sed,
    Sei,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
}

#[derive(Clone, Copy)]
pub struct OpInfo {
    pub address_mode: AddrMode,
    pub instruction: Instruction,
}

#[derive(Clone, Copy)]
pub struct Op {
    pub address_mode: fn(cpu: &mut Cpu),
    pub instruction: fn(cpu: &mut Cpu),
    pub cycles: u8,
    pub info: OpInfo,
}

use crate::mmio;
pub struct Cpu {
    pub regs: Regs,

    pub offset: i8,
    pub addr: u16,
    pub operand: u8,
    pub cycles: u8,

    pub branch: bool,
    pub page_crossed: bool,

    // we need to store the addressing modes
    // because for some stupid reason the
    // ACC mode exists and forces us to
    // differentiate between writing to memory
    // vs. writing the A register within the
    // context of a single instruction
    pub addr_mode: AddrMode,

    pub mmio: mmio::Mmio,
    opcodes: Vec<Op>,
}

macro_rules! bit {
    ($x: expr, $n: expr) => {
        ($x & (1 << $n) > 0)
    };
}
pub(crate) use bit;

// we compare (XOR) the carries (c6, c7) from bits 6 and 7
fn check_add_overflow(x: u8, y: u8) -> bool {
    let c6 = bit!(x, 6) & bit!(y, 6);
    let c7 = bit!(x, 7) & bit!(y, 7);
    c6 ^ c7
}

fn is_neg(x: u8) -> bool {
    bit!(x, 7)
}

impl Cpu {
    pub fn execute(&mut self, opcode: u8) {
        let op = &self.opcodes[opcode as usize].clone();
        self.addr_mode = op.info.address_mode;
        (op.address_mode)(self);
        (op.instruction)(self);

        if self.branch {
            self.cycles += 1 + self.page_crossed as u8;
            self.regs.pc = self.regs.pc.wrapping_add(self.offset as u16);
        }

        self.cycles += op.cycles;
    }

    fn read_inst(&mut self) -> u8 {
        let inst = self.mmio.read_byte(self.regs.pc);
        self.regs.pc += 1;
        inst
    }

    fn read_data(&self) -> u8 {
        self.mmio.read_byte(self.addr)
    }

    pub fn step(&mut self) {
        let inst = self.read_inst();
        self.execute(inst);
    }

    pub fn new() -> Cpu {
        let nop = Op {
            address_mode: Cpu::imp,
            instruction: Cpu::nop,
            cycles: 1,
            info: OpInfo {
                address_mode: AddrMode::Imp,
                instruction: Instruction::Nop,
            },
        };

        let cpu = Cpu {
            regs: Regs::default(),

            offset: 0x0,
            addr: 0x0,
            operand: 0x0,
            cycles: 0,
            page_crossed: false,
            branch: false,
            addr_mode: AddrMode::Imm,

            mmio: mmio::Mmio::new(),
            opcodes: [nop; 0x100].to_vec(),
        };

        return cpu;
    }

    // Addressing modes

    pub fn imp(&mut self) {
        self.page_crossed = false;
    }

    pub fn acc(&mut self) {
        self.page_crossed = false;
    }

    pub fn imm(&mut self) {
        self.operand = self.read_inst();
        self.page_crossed = false;
    }

    pub fn abs(&mut self) {
        self.addr = self.read_inst() as u16;
        self.addr += (self.read_inst() as u16) << 8;
        self.operand = self.read_data();
        self.page_crossed = false;
    }

    pub fn zp(&mut self) {
        self.addr = self.read_inst() as u16;
        self.operand = self.read_data();
        self.page_crossed = false;
    }

    pub fn zpx(&mut self) {
        self.addr = self.read_inst() as u16;
        self.addr += self.regs.x as u16;
        // zero-page addrmodes wrap within zero page
        self.addr &= 0x00FF;
        self.operand = self.read_data();
        self.page_crossed = false;
    }

    pub fn zpy(&mut self) {
        self.addr = self.read_inst() as u16;
        self.addr += self.regs.y as u16;
        // zero-page addrmodes wrap within zero page
        self.addr &= 0x00FF;
        self.operand = self.read_data();
        self.page_crossed = false;
    }

    pub fn absx(&mut self) {
        self.addr = self.read_inst() as u16;
        self.addr += (self.read_inst() as u16) << 8;
        self.addr += self.regs.x as u16;
        self.operand = self.read_data();
        self.page_crossed = self.addr & 0xFF00 != 0;
    }

    pub fn absy(&mut self) {
        self.addr = self.read_inst() as u16;
        self.addr += (self.read_inst() as u16) << 8;
        self.addr += self.regs.y as u16;
        self.operand = self.read_data();
        self.page_crossed = self.addr & 0xFF00 != 0;
    }

    pub fn indx(&mut self) {
        self.addr = self.read_inst() as u16;
        self.addr += self.regs.x as u16;
        self.addr &= 0x00FF;
        let lo = self.read_data() as u16;
        let hi = self.read_inst() as u16;
        self.addr = (hi << 8) + lo;
        self.operand = self.read_data();
        self.page_crossed = false;
    }

    pub fn indy(&mut self) {
        self.addr = self.read_inst() as u16;
        let mut lo = self.read_data() as u16;
        lo += self.regs.y as u16;
        self.page_crossed = lo & 0xFF00 != 0;
        let hi = self.read_inst() as u16;
        self.addr = (hi << 8) + lo;
        self.operand = self.read_data();
    }

    pub fn ind(&mut self) {
        let ptr_lo = self.read_inst() as u16;
        let ptr_hi = self.read_inst() as u16;
        self.addr = (ptr_hi << 8) + ptr_lo;

        let mut pc = self.read_data() as u16;
        // accounts for the JMP bug
        if ptr_lo == 0xFF {
            self.addr &= 0xFF00;
        } else {
            self.addr += 1;
        }
        pc += (self.read_data() as u16) << 8;
        self.regs.pc = pc;
        self.page_crossed = false;
    }

    pub fn rel(&mut self) {
        self.offset = self.read_inst() as i8;
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        let mut p = self.regs.p;
        let mask = (value as u8) << (flag as u8);
        p = (p & !mask) | mask;
        self.regs.p = p;
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        bit!(self.regs.p, flag as u8)
    }

    pub fn update_nz_flags(&mut self) {
        self.set_flag(Flag::N, is_neg(self.regs.a));
        self.set_flag(Flag::Z, self.regs.a == 0);
    }

    // Instructions

    /*
        Special
    */
    pub fn nop(&mut self) {}

    pub fn brk(&mut self) {
        unimplemented!();
    }

    /*
        Arithmetic
    */

    pub fn adc(&mut self) {
        let a = self.regs.a;
        let a16 = a as u16;
        let m = self.operand;
        let m16 = m as u16;
        let c16 = self.get_flag(Flag::C) as u16;

        let res16 = a16 + m16 + c16;
        let res8 = res16 as u8;
        self.regs.a = res8;

        // check if both operands have sign bits that are
        // different from the result
        let v = ((a ^ res8) & (m ^ res8) & 0x80) > 0;
        self.set_flag(Flag::V, v);

        self.update_nz_flags();
        self.set_flag(Flag::C, (res16 & 0xFF00) > 0);
    }

    pub fn sbc(&mut self) {
        // (for 8 bits)
        // SUBC = A - M - (1-C)
        // = A - M - (1-C) + 256
        // = A - (M - 255) + C = A + !M + C
        let a = self.regs.a;
        let a16 = a as u16;
        let m = self.operand;
        let not_m = !m;
        let not_m16 = not_m as u16;
        let c16: u16 = !self.get_flag(Flag::C) as u16;

        let res16 = a16 + not_m16 + c16;
        let res8 = res16 as u8;
        self.regs.a = res8;

        let v = ((a ^ res8) & (not_m ^ res8) & 0x80) > 0;
        self.set_flag(Flag::V, v);

        self.update_nz_flags();
        self.set_flag(Flag::C, (res16 & 0xFF00) > 0);
    }

    pub fn cmp(&mut self) {
        // tests A-M
        let a = self.regs.a;
        let m = self.operand;
        self.set_flag(Flag::Z, a == m);
        self.set_flag(Flag::C, a >= m);
        self.set_flag(Flag::N, a < m);
    }

    pub fn cpx(&mut self) {
        // tests X-M
        let a = self.regs.a;
        let x = self.regs.x;
        self.set_flag(Flag::Z, a == x);
        self.set_flag(Flag::C, a >= x);
        self.set_flag(Flag::N, a < x);
    }

    pub fn cpy(&mut self) {
        // tests Y-M
        let a = self.regs.a;
        let y = self.regs.y;
        self.set_flag(Flag::Z, a == y);
        self.set_flag(Flag::C, a >= y);
        self.set_flag(Flag::N, a < y);
    }

    /*
        Shifts
    */

    pub fn asl(&mut self) {
        let res: u16 = (self.regs.a as u16) * 2;
        self.regs.a = res as u8;

        self.set_flag(Flag::V, res & 0xFF > 0);
        self.update_nz_flags();
    }

    /*
        Logical
    */

    pub fn and(&mut self) {
        self.regs.a &= self.operand;
        self.update_nz_flags();
        if self.page_crossed {
            self.cycles += 1;
        }
    }

    /*
        Branches
    */

    pub fn bcc(&mut self) {
        self.branch = !self.get_flag(Flag::C);
    }

    pub fn bcs(&mut self) {
        self.branch = self.get_flag(Flag::C);
    }

    pub fn beq(&mut self) {
        self.branch = self.get_flag(Flag::Z);
    }

    pub fn bne(&mut self) {
        self.branch = !self.get_flag(Flag::Z);
    }

    pub fn bit(&mut self) {
        let m = self.operand;
        let res = self.regs.a & m;
        self.set_flag(Flag::Z, res == 0);
        self.set_flag(Flag::V, bit!(m, 6));
        self.set_flag(Flag::N, bit!(m, 7));
    }

    pub fn bmi(&mut self) {
        self.branch = self.get_flag(Flag::N);
    }

    pub fn bpl(&mut self) {
        self.branch = !self.get_flag(Flag::N);
    }

    pub fn bvc(&mut self) {
        self.branch = !self.get_flag(Flag::V);
    }

    pub fn bvs(&mut self) {
        self.branch = self.get_flag(Flag::V);
    }

    /*
        Status flag changes
    */

    pub fn clc(&mut self) {
        self.set_flag(Flag::C, false);
    }

    pub fn cld(&mut self) {
        self.set_flag(Flag::D, false);
    }

    pub fn cli(&mut self) {
        self.set_flag(Flag::I, false);
    }

    pub fn clv(&mut self) {
        self.set_flag(Flag::V, false);
    }

    pub fn dec(&mut self) {}

    pub fn dex(&mut self) {}

    pub fn dey(&mut self) {}

    pub fn eor(&mut self) {}

    pub fn inc(&mut self) {}

    pub fn inx(&mut self) {}

    pub fn iny(&mut self) {}

    pub fn jmp(&mut self) {}

    pub fn jsr(&mut self) {}

    pub fn lda(&mut self) {}

    pub fn ldx(&mut self) {
        self.regs.x = self.operand;
        self.update_nz_flags();
    }

    pub fn ldy(&mut self) {}

    pub fn lsr(&mut self) {}

    pub fn ora(&mut self) {}

    pub fn pha(&mut self) {}

    pub fn php(&mut self) {}

    pub fn pla(&mut self) {}

    pub fn plp(&mut self) {}

    pub fn rol(&mut self) {}

    pub fn ror(&mut self) {}

    pub fn rti(&mut self) {}

    pub fn rts(&mut self) {}

    pub fn sec(&mut self) {}

    pub fn sed(&mut self) {}

    pub fn sei(&mut self) {}

    pub fn sta(&mut self) {}

    pub fn stx(&mut self) {}

    pub fn sty(&mut self) {}

    pub fn tax(&mut self) {}

    pub fn tay(&mut self) {}

    pub fn tsx(&mut self) {}

    pub fn txa(&mut self) {}

    pub fn txs(&mut self) {}

    pub fn tya(&mut self) {}
}
