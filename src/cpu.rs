use mem::Mem;

#[derive(Debug)]
pub struct Cpu {
    pc : u16,
    sp : u8,
    reg : [u8; 16],
    stack : [u16; 16],
    reg_i : u16,
    //ins : u16,
    memory : Mem
}

impl Cpu {
    pub fn new() -> Cpu {
        let ret_val = Cpu {
            pc : 0x200,
            sp : 0,
            reg : [0u8; 16],
            stack : [0u16; 16],
            reg_i : 0,
            //ins : 0,
            memory : Default::default()
        };
        ret_val
    }
    pub fn exec_instruction(&mut self) -> Result<(), String>{

        let ins = self.get_next_instruction();
        //instructions are decoded as so
        // abcd
        // _nnn
        // jj__
        // __kk
        let a   = ((ins>>12)   )  as u8;
        let b   = ((ins>>8)&0xf)  as u8;
        let c   = ((ins>>4)&0xf)  as u8;
        let d   = ((ins   )&0xf)  as u8;
        let jj  = (ins>> 8)  as u8;
        let kk  = (ins&0xff) as u8;
        let nnn = ins&0xfff;

        match a {
            //TODO: Implement screen clear, Return
            0x0 => match kk {
                0xe0 => unimplemented!(), //TODO: Clear display
                0xee => unimplemented!(), //TODO: return subroutine
                _   => return Err(format!("Unknown Opcode encountered: 0x{:x}", ins)),
            },
            0x1 => {
                //1nnn - JP addr
                //Jump to location nnn.
                self.pc=nnn;
                return Ok(());
            },
            0x2 => unimplemented!(), //TODO: CALL addr
            0x3 => {
                //SE Vx, byte
                //Skip next instruction if Vx = kk.
                if self.reg[b as usize] == kk {
                    self.pc += 2;
                }
            },
            0x4 => {
                //SNE Vx, byte
                //Skip next instruction if Vx != kk.
                if self.reg[b as usize] != kk {
                    self.pc += 2;
                }
            },
            0x5 => { 
                //5xy0 - SE Vx, Vy
                //Skip next instruction if Vx = Vy.
                if self.reg[b as usize] == self.reg[c as usize] {
                    self.pc += 2;
                }
            },
            0x6 => { 
                //6xkk - LD Vx, byte
                //Set Vx = kk.
                self.reg[b as usize] = kk;
            },
            0x7 => { 
                //7xkk - ADD Vx, byte
                //Set Vx = Vx + kk.
                self.reg[b as usize]+= kk;
            },
            0x8 => match d { 
                0x0 => {
                    //8xy0 - LD Vx, Vy
                    //Set Vx = Vy.
                    self.reg[b as usize] = self.reg[c as usize];
                },
                0x1 => {
                    //8xy1 - OR Vx, Vy
                    //Set Vx = Vx OR Vy.
                    self.reg[b as usize] |= self.reg[c as usize];
                },
                0x2 => {
                    //8xy2 - AND Vx, Vy
                    //Set Vx = Vx AND Vy.
                    self.reg[b as usize] &= self.reg[c as usize];
                },
                0x3 => {
                    //8xy3 - XOR Vx, Vy
                    //Set Vx = Vx XOR Vy.
                    self.reg[b as usize] ^= self.reg[c as usize];
                },
                0x4 => {
                    //8xy4 - ADD Vx, Vy
                    //Set Vx = Vx + Vy, set VF = carry.
                    let sum : u16 = 
                        self.reg[b as usize] as u16 + self.reg[c as usize] as u16;
                    self.reg[0xf] = (sum > 255) as u8;
                    self.reg[b as usize] = sum as u8;
                },
                0x5 => {
                    //8xy5 - SUB Vx, Vy
                    //Set Vx = Vx - Vy, set VF = (Vx > Vy).
                    let sum : i16 = 
                        self.reg[b as usize] as i16 - self.reg[c as usize] as i16;
                    self.reg[0xf] = (sum > 0) as u8;
                    self.reg[b as usize] = sum as u8;
                },
                0x6 => {
                    //8xy6 - SHR Vx {, Vy}
                    //Set Vx = Vx SHR 1, set VF = Vx[LSB]
                    self.reg[0xf] = self.reg[b as usize] & 0x01;
                    self.reg[b as usize]>>=1;
                },
                0x7 => {
                    //8xy7 - SUBN Vx, Vy
                    //Set Vx = Vy - Vx, set VF = (Vy > Vx).
                    let sum : i16 = 
                        self.reg[c as usize] as i16 - self.reg[b as usize] as i16;
                    self.reg[0xf] = (sum > 0) as u8;
                    self.reg[b as usize] = sum as u8;
                },
                0xE => {
                    //8xyE - SHL Vx {, Vy}
                    //Set Vx = Vx SHL 1, set VF = Vx[MSB]
                    self.reg[0xf] = (self.reg[b as usize] & 0x80u8 != 0) as u8;
                    self.reg[b as usize]<<=1;
                },
                _   => return Err(format!("Unknown Opcode encountered: 0x{:x}", ins)),
            },
            0x9 => { 
                //9xy0 - SNE Vx, Vy
                //Skip next instruction if Vx != Vy.
                if self.reg[b as usize] != self.reg[c as usize] {
                    self.pc += 2;
                }
            },
            0xA => {
                //Annn - LD I, addr
                //Set I = nnn.
                self.reg_i=nnn;
            },
            0xB => {
                //Bnnn - JP V0, addr
                //Jump to location nnn + V0.
                self.pc=nnn+(self.reg[0] as u16);
                return Ok(());
            },
            _ => return Err(format!("Unknown Opcode encountered: 0x{:x}", ins)),
        };

        self.pc += 2;
        Ok(())
    }
    fn get_next_instruction(&mut self) -> u16 {
        self.memory.read_u16(self.pc as usize)
    }
}

fn attempt(obj: Result<(), String>){
    match obj {
        Err(v) => panic!(v),
        _   => (),
    }
}

#[test]
fn test_exec_instruction(){
    {   //test 1nnn - JP addr
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, &vec![0x14, 0xff]);
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x4ff {
            panic!("Test failed for ins 1nnn");
        }
    }
    {   //test 6xkk - LD Vx, byte
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, &vec![0x60, 0xea]);
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xea {
            panic!("Test failed for ins 6xkk");
        }
    }
    {   //test 3xkk - SE Vx, byte
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x11, 0x30, 0xea, 0x30, 0x11]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x204 {
            panic!("Test failed for ins 3xkk on branch not taken path");
        }
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x208 {
            panic!("Test failed for ins 3xkk on branch taken path");
        }
    }
    {   //test 4xkk - SNE Vx, byte
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x11, 0x40, 0x11, 0x40, 0xea]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x204 {
            panic!("Test failed for ins 4xkk on branch not taken path");
        }
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x208 {
            panic!("Test failed for ins 4xkk on branch taken path");
        }
    }
    {   //test 5xy0 - SE Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x61, 0x11, 0x50, 0x10, 0x50, 0x20]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x204 {
            panic!("Test failed for ins 5xy0 on branch not taken path");
        }
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x208 {
            panic!("Test failed for ins 5xy0 on branch taken path");
        }
    }
    {   //test 7xkk - ADD Vx, byte
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, &vec![0x70, 0xea, 0x70, 0x11]);
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xea {
            panic!("Test failed for ins 7xkk");
        }
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xfb {
            panic!("Test failed for ins 7xkk");
        }
    }
    {   //test 8xy0 - LD Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x61, 0x11, 0x80, 0x10]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != cpu.reg[1] {
            panic!("Test failed for ins 8xy0");
        }
    }
    {   //test 8xy1 - OR Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x22, 0x61, 0x11, 0x80, 0x11]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x33 {
            panic!("Test failed for ins 8xy1");
        }
    }
    {   //test 8xy2 - AND Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x33, 0x61, 0x11, 0x80, 0x12]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x11 {
            panic!("Test failed for ins 8xy2");
        }
    }
    {   //test 8xy3 - XOR Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x33, 0x61, 0x11, 0x80, 0x13]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x22 {
            panic!("Test failed for ins 8xy3");
        }
    }
    {   //test 8xy4 - ADD Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x33, 0x61, 0xcc, 0x80, 0x14, 0x80, 0x14]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xff || cpu.reg[15] != 0 {
            panic!("Test failed for ins 8xy4 (Non-overflow)");
        }
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xcb || cpu.reg[15] != 1 {
            panic!("Test failed for ins 8xy4 (Overflow)");
        }
    }
    {   //test 8xy5 - SUB Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x33, 0x61, 0x22, 0x80, 0x15, 0x80, 0x15]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x11 || cpu.reg[15] != 1 {
            panic!("Test failed for ins 8xy5 (Non-overflow)");
        }
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xef || cpu.reg[15] != 0 {
            panic!("Test failed for ins 8xy5 (Overflow)");
        }
    }
    {   //test 8xy6 - SHR Vx {, Vy}
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x06, 0x80, 0x16, 0x80, 0x16]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x03 || cpu.reg[15] != 0 {
            panic!("Test failed for ins 8xy6 (Non-carry)");
        }
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x01 || cpu.reg[15] != 1 {
            panic!("Test failed for ins 8xy6 (Carry)");
        }
    }
    {   //test 8xy7 - SUBN Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x11, 0x61, 0x22, 0x80, 0x17, 0x60, 0x33, 0x80, 0x17]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x11 || cpu.reg[15] != 1 {
            panic!("Test failed for ins 8xy7 (Non-overflow)");
        }
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xef || cpu.reg[15] != 0 {
            panic!("Test failed for ins 8xy7 (Overflow)");
        }
    }
    {   //test 8xyE - SHL Vx {, Vy}
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x60, 0x80, 0x1e, 0x80, 0x1e]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xc0 || cpu.reg[15] != 0 {
            panic!("Test failed for ins 8xyE (Non-carry)");
        }
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x80 || cpu.reg[15] != 1 {
            panic!("Test failed for ins 8xyE (Carry)");
        }
    }
    {   //test 9xy0 - SNE Vx, Vy
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x61, 0x11, 0x90, 0x20, 0x90, 0x10]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x204 {
            panic!("Test failed for ins 9xy0 on branch not taken path");
        }
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x208 {
            panic!("Test failed for ins 9xy0 on branch taken path");
        }
    }
    {   //test Annn - LD I, addr
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0xA1, 0x23]);
        attempt(cpu.exec_instruction());
        if cpu.reg_i != 0x123 {
            panic!("Test failed for ins Annn");
        }
    }
    {   //test Bnnn - JP V0, addr
        let mut cpu = Cpu::new();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x02, 0xB2, 0x23]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x225 {
            panic!("Test failed for ins Bnnn");
        }
    }
}
