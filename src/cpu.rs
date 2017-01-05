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
    pub fn exec_instruction(&mut self) -> Result<(), &'static str>{

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
            0x1 => {
                //1nnn - JP addr
                //Jump to location nnn.
                self.pc=nnn;
                return Ok(());
            },
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
                    if sum > 255 {
                        self.reg[0xf] = 1;
                    }
                    self.reg[b as usize] = sum as u8;
                },
                _   => return Err("Unknown Opcode encountered!"),
            },
            _ => return Err("Unknown Opcode encountered!"),
        };

        self.pc += 2;
        Ok(())
    }
    fn get_next_instruction(&mut self) -> u16 {
        self.memory.read_u16(self.pc as usize)
    }
}

fn attempt(obj: Result<(), &'static str>){
    match obj {
        Err(v) => panic!(v),
        _   => (),
    }
}

#[test]
fn test_exec_instruction(){
    {   //test 1nnn - JP addr
        let mut cpu = Cpu::new();
        cpu.pc=0x200u16;
        cpu.memory.memset(0x200, &vec![0x14, 0xff]);
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x4ff {
            panic!("Test failed for ins 1nnn");
        }
    }
    {   //test 6xkk - LD Vx, byte
        let mut cpu = Cpu::new();
        cpu.pc=0x200u16;
        cpu.memory.memset(0x200, &vec![0x60, 0xea]);
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xea {
            panic!("Test failed for ins 6xkk");
        }
    }
    {   //test 3xkk - SE Vx, byte
        let mut cpu = Cpu::new();
        cpu.pc=0x200u16;
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
        cpu.pc=0x200u16;
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
        cpu.pc=0x200u16;
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
        cpu.pc=0x200u16;
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
        cpu.pc=0x200u16;
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
        cpu.pc=0x200u16;
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
        cpu.pc=0x200u16;
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
        cpu.pc=0x200u16;
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
        cpu.pc=0x200u16;
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
}
