extern crate rand;
use mem::Mem;
use rand::Rng;

#[derive(Debug)]
pub struct Cpu {
    pc : u16,
    sp : usize,
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
        //let jj  = (ins>> 8)  as u8;
        let kk  = (ins&0xff) as u8;
        let nnn = ins&0xfff;

        //err message for unknown opcode
        let err_unknown_opcode = Err(format!("Unknown Opcode encountered: 0x{:x}", ins)); 

        match a {
            0x0 => match kk {
                0xe0 => { //TODO: Test this
                    //00E0 - CLS
                    //Clear the display.
                    self.memory.clear_screen()
                },
                0xee => {
                    //00EE - RET
                    //Return from a subroutine.
                    self.pc = try!(self.pop_stack());
                    return Ok(());
                },
                _   => return err_unknown_opcode,
            },
            0x1 => {
                //1nnn - JP addr
                //Jump to location nnn.
                self.pc=nnn;
                return Ok(());
            },
            0x2 => {
                //2nnn - CALL addr
                //Call subroutine at nnn.
                let pc = self.pc;
                try!(self.push_stack(pc+2u16));
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
                _   => return err_unknown_opcode,
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
            0xC => {
                //Bnnn - JP V0, addr
                //Jump to location nnn + V0.
                self.reg[b as usize]=kk & (rand::thread_rng().gen_range(0,256) as u8);
                return Ok(());
            },
            0xD => unimplemented!(), //TODO: Dxyn - DRW Vx, Vy, nibble
            0xE => match kk {
                0x9E => unimplemented!(), //TODO: Ex9E - SKP Vx
                0xA1 => unimplemented!(), //TODO: ExA1 - SKNP Vx
                _ => return err_unknown_opcode,
            },
            0xF => match kk {
                0x07 => unimplemented!(), //TODO: Fx07 - LD Vx, DT
                0x0A => unimplemented!(), //TODO: Fx0A - LD Vx, K
                0x15 => unimplemented!(), //TODO: Fx15 - LD DT, Vx
                0x18 => unimplemented!(), //TODO: Fx18 - LD ST, Vx
                0x1E => {
                    //Fx1E - ADD I, Vx
                    //Set I = I + Vx.
                    self.reg_i+=self.reg[b as usize] as u16;
                }, 
                0x29 => {
                    //Fx29 - LD F, Vx
                    //Set I = location of sprite for digit Vx.
                    self.reg_i=Mem::get_sprite_addr(self.reg[b as usize]&0xfu8);
                },
                0x33 => {
                    //Fx33 - LD B, Vx
                    //Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    let value = self.reg[b as usize];
                    let addr = self.reg_i as usize;
                    self.memory.write_u8(  addr, value/100);
                    self.memory.write_u8(addr+1, (value/10)%10);
                    self.memory.write_u8(addr+2, value%10);
                }, 
                0x55 => {
                    //Fx55 - LD [I], Vx TODO
                    //Store registers V0 through Vx in memory starting at location I.
                    let addr = self.reg_i as usize;
                    let vec = &self.reg[0..(((b&0xf)+1) as usize)].to_vec();
                    self.memory.memset(addr, vec);
                }, 
                0x65 => unimplemented!(), //Fx65 - LD Vx, [I] TODO
                _ => return err_unknown_opcode,
            },
            _ => unreachable!()
        };

        self.pc += 2;
        Ok(())
    }
    pub fn reset(&mut self){
        self.memory.reset();
        self.pc = 0x200;
        self.sp = 0;
        self.reg = [0u8; 16];
        self.stack = [0u16; 16];
        //self.ins = 0;
        self.reg_i = 0;
    }
    fn get_next_instruction(&mut self) -> u16 {
        self.memory.read_u16(self.pc as usize)
    }
    fn push_stack(&mut self, val: u16) -> Result<(), String> {
        if self.sp >= 16 {
            return Err(format!("Progam stack overflowed! {:?}", self.stack));
        }
        self.stack[self.sp]=val;
        self.sp+=1;
        Ok(())
    }
    fn pop_stack(&mut self) -> Result<u16, String> {
        if self.sp == 0 {
            return Err(format!("Progam stack underflowed!"));
        }
        self.sp-=1;
        Ok(self.stack[self.sp])
    }
}

fn attempt<T>(obj: Result<T, String>) -> T {
    match obj {
        Err(e) => panic!(e),
        Ok(v)  => v,
    }
}

//TESTS

#[test]
fn test_exec_instruction(){
    let mut cpu = Cpu::new();
    {   //test 1nnn - JP addr
        cpu.reset();
        cpu.memory.memset(0x200, &vec![0x14, 0xff]);
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x4ff {
            panic!("Test failed for ins 1nnn");
        }
    }
    {   //test 2nnn - CALL addr
        cpu.reset();
        cpu.memory.memset(0x200, &vec![0x24, 0xff]);
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x4ff || cpu.stack[0] != 0x202 || cpu.sp != 1 {
            panic!("Test failed for ins 2nnn");
        }
    }
    {   //test 00EE - RET
        cpu.reset();
        cpu.memory.memset(0x200, &vec![0x24, 0x00]);
        cpu.memory.memset(0x400, &vec![0x00, 0xee]);
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x400 || cpu.stack[0] != 0x202 || cpu.sp != 1 {
            panic!("Ins 2nnn failed in test for ins 00ee?");
        }
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x202 || cpu.sp != 0 {
            panic!("Test failed for ins 00ee");
        }
    }
    {   //test 6xkk - LD Vx, byte
        cpu.reset();
        cpu.memory.memset(0x200, &vec![0x60, 0xea]);
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0xea {
            panic!("Test failed for ins 6xkk");
        }
    }
    {   //test 3xkk - SE Vx, byte
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x61, 0x11, 0x80, 0x10]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != cpu.reg[1] {
            panic!("Test failed for ins 8xy0");
        }
    }
    {   //test 8xy1 - OR Vx, Vy
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
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
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0xA1, 0x23]);
        attempt(cpu.exec_instruction());
        if cpu.reg_i != 0x123 {
            panic!("Test failed for ins Annn");
        }
    }
    {   //test Bnnn - JP V0, addr
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x02, 0xB2, 0x23]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x225 {
            panic!("Test failed for ins Bnnn");
        }
    }
    {   //test Cxkk - RND Vx, byte
        //no real way to test randomness, so just ensure 
        //that instruction is decoded and bitmask works
        const NUM_TRIALS : i32 = 100;
        cpu.reset();
        for bitmask in 0..256 {
            let bitmask = bitmask as u8;
            cpu.memory.memset(0x200, 
                &vec![0xC0, bitmask]);
            for _ in 0..NUM_TRIALS {
                cpu.pc=0x200;
                attempt(cpu.exec_instruction());
                if cpu.reg[0] & (!bitmask) != 0 { //ie the bitmask isn't adhered to
                    panic!("Test failed for ins Cxkk: bitmask didn't work");
                }
            }
        }
    }
    {   //test Fx1E - ADD I, Vx
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x04, 0xA2, 0x23, 0xF0, 0x1E]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg_i != 0x227 {
            panic!("Test failed for ins Fx1E");
        }
    }
    {   //Fx29 - LD F, Vx
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x63, 0x0f, 0xf3, 0x29]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if  cpu.memory.read_u8((cpu.reg_i  ) as usize) != 0xf0 ||
            cpu.memory.read_u8((cpu.reg_i+1) as usize) != 0x80 ||
            cpu.memory.read_u8((cpu.reg_i+2) as usize) != 0xf0 ||
            cpu.memory.read_u8((cpu.reg_i+3) as usize) != 0x80 ||
            cpu.memory.read_u8((cpu.reg_i+4) as usize) != 0x80 {
            panic!("Test failed for ins Fx29");
        }
    }
    {   //Fx33 - LD B, Vx
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x60, 123, 0xA4, 0x00, 0xF0, 0x33]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if  cpu.memory.read_u8(0x400) != 1 ||
            cpu.memory.read_u8(0x401) != 2 ||
            cpu.memory.read_u8(0x402) != 3 {
            panic!("Test failed for ins Fx33");
        }
    }
    {   //Fx55 - LD [I], Vx
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x60, 1, 0x61, 2, 0x62, 3, 0xA4, 0x00, 0xF2, 0x55]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if  cpu.memory.read_u8(0x400) != 1 ||
            cpu.memory.read_u8(0x401) != 2 ||
            cpu.memory.read_u8(0x402) != 3 {
            panic!("Test failed for ins Fx55");
        }
    }
}
