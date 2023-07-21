pub struct Mmio {
    // ideally we should have several arrays here
    // representing the memory mapped physical devices
    pub ram: Vec<u8>,
    pub rom: Vec<u8>,
}

impl Mmio {
    pub fn new() -> Mmio {
        Mmio {
            ram: vec![0; 0x0800], // $0000 to $07FF
            rom: vec![0; 0x8000], // $8000 to $FFFF
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        if addr <= 0x07FF {
            return self.ram[addr as usize];
        } else if 0x8000 <= addr {
            return self.rom[(addr - 0x8000) as usize];
        } else {
            return 0x0;
        }
    }

    pub fn read(&self, addr: u16, size: u16) -> Vec<u8> {
        let start = addr as usize;
        let end = addr as usize + size as usize;
        self.ram[start..end].to_vec()
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        if addr <= 0x07FF {
            self.ram[addr as usize] = byte;
        } else if 0x8000 <= addr {
            self.rom[(addr - 0x8000) as usize] = byte;
        }
    }

    pub fn write(&mut self, addr: u16, bytes: &[u8]) {
        for (i, &byte) in bytes.iter().enumerate() {
            self.write_byte(addr + i as u16, byte);
        }
    }
}
