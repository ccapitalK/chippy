extern crate rand;
use mem::Mem;
use rand::Rng;

const MIN_INS_PER_SECOND: u64 = 60u64;
const MAX_INS_PER_SECOND: u64 = 4000u64;
pub const DEFAULT_INS_PER_SECOND: u64 = 400u64;

pub struct Cpu {
    pc:    u16,
    sp:    usize,
    dt:    u8,
    st:    u8,
    reg:   [u8; 16],
    stack: [u16; 16],
    reg_i: u16,
    pub memory : Mem,

    keys:  [bool; 16],
    key_counters: [u16; 16],
    instructions_per_second: u64,
}

const KEY_TRAIL_LENGTH: u16 = 150;

impl Cpu { 
    pub fn new() -> Cpu {
        let ret_val = Cpu {
            pc:    0x200,
            sp:    0,
            dt:    0,
            st:    0,
            reg:   [0u8; 16],
            stack: [0u16; 16],
            reg_i: 0,
            memory : Default::default(),
            keys:  [false; 16],
            key_counters: [0u16; 16],
            instructions_per_second: DEFAULT_INS_PER_SECOND,
        };
        ret_val
    }
    #[allow(dead_code)]
    pub fn print_instruction(&self){
        println!("[0x{:03x}]: {:04X}", self.pc, self.get_next_instruction());
    }
    pub fn exec_instruction(&mut self) -> Result<(), String>{

        for i in self.key_counters.iter_mut() {
            *i=i.saturating_sub(1u16);
        }

        if self.pc >= 0x1000 {
            return Err(format!("PC at illegal address: {}", self.pc));
        }
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
        let err_unknown_opcode = Err(format!("Unknown instruction encountered at address[0x{:03x}]: 0x{:04x}", self.pc, ins)); 

        match a {
            0x0 => match kk {
                0xe0 => {
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
                self.reg[b as usize] = self.reg[b as usize].wrapping_add(kk);
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
                //Cxkk - RND Vx, byte
                //Set Vx = random byte AND kk.
                self.reg[b as usize]=kk & (rand::thread_rng().gen_range(0,256) as u8);
            },
            0xD => {
                //Dxyn - DRW Vx, Vy, nibble
                //Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                let x = self.reg[b as usize];
                let y = self.reg[c as usize];
                self.reg[0xf]=self.memory
                    .draw_sprite(self.reg_i, x, y, d) as u8;
            }, 
            0xE => match kk {
                0x9E => {
                    //Ex9E - SKP Vx
                    //Skip next instruction if key with the value of Vx is pressed.
                    let key = self.reg[b as usize]&0xfu8;
                    self.key_counters[key as usize]=KEY_TRAIL_LENGTH;
                    if self.keys[key as usize] {
                        self.pc+=2;
                    }
                },
                0xA1 => {
                    //ExA1 - SKNP Vx
                    //Skip next instruction if key with the value of Vx is not pressed.
                    let key = self.reg[b as usize]&0xfu8;
                    self.key_counters[key as usize]=KEY_TRAIL_LENGTH;
                    if !self.keys[key as usize] {
                        self.pc+=2;
                    }
                },
                _ => return err_unknown_opcode,
            },
            0xF => match kk {
                0x07 => {
                    //Fx07 - LD Vx, DT
                    //Set Vx = delay timer value.
                    self.reg[b as usize]=self.dt;
                }, 
                0x0A => {
                    //Fx0A - LD Vx, K
                    //Wait for a key press, store the value of the key in Vx.
                    self.reg[b as usize]=0xff;
                    for i in 0..16 {
                        if self.keys[i] {
                            self.reg[b as usize] = i as u8;
                            break;
                        }
                    }
                    if self.reg[b as usize] == 0xff {
                        return Ok(());
                    }
                },
                0x15 => {
                    //Fx15 - LD DT, Vx
                    //Set delay timer = Vx.
                    self.dt=self.reg[b as usize];
                }, 
                0x18 => {
                    //Fx18 - LD ST, Vx
                    //Set sound timer = Vx.
                    self.st=self.reg[b as usize];
                }, 
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
                    //Fx55 - LD [I], Vx
                    //Store registers V0 through Vx in memory starting at location I.
                    let addr = self.reg_i as usize;
                    let vec = &self.reg[0..((b+1) as usize)].to_vec();
                    self.memory.memset(addr, vec);
                }, 
                0x65 => {
                    //Fx65 - LD Vx, [I] 
                    //Read registers V0 through Vx from memory starting at location I.
                    let addr = self.reg_i as usize;
                    let b = b as usize;
                    let mem_vec = self.memory.get_vec(addr, b+1);
                    self.reg[0..b+1].clone_from_slice(mem_vec.as_slice());
                }, 
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
        self.dt = 0;
        self.st = 0;
        self.reg = [0u8; 16];
        self.stack = [0u16; 16];
        //don't modify keys
        self.reg_i = 0;
    }
    fn get_next_instruction(&self) -> u16 {
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
    pub fn decr_dt(&mut self) {
        self.dt = self.dt.saturating_sub(1u8);
    }
    pub fn decr_st(&mut self) -> bool {
        self.st = self.st.saturating_sub(1u8);
        self.st == 0u8
    }
    pub fn keydown(&mut self, keycode: u8) {
        self.keys[keycode as usize]=true;
    }
    pub fn keyup(&mut self, keycode: u8) {
        self.keys[keycode as usize]=false;
    }
    pub fn get_key(&self, keycode: u8) -> bool {
        self.keys[keycode as usize]
    }
    pub fn get_key_trail(&self, keycode: u8) -> u8 {
        let prod: u32 = (self.key_counters[keycode as usize] as u32)*256;
        (prod/(KEY_TRAIL_LENGTH as u32)) as u8
    }
    pub fn increase_ips(&mut self) {
        use std::cmp::min;
        self.instructions_per_second 
            = min(self.instructions_per_second+10, MAX_INS_PER_SECOND);
    }
    pub fn decrease_ips(&mut self) {
        use std::cmp::max;
        self.instructions_per_second 
            = max(self.instructions_per_second-10, MIN_INS_PER_SECOND);
    }
    pub fn get_ips(&self) -> u64 {
        self.instructions_per_second
    }
    pub fn set_ips(&mut self, new_ips: u64) {
        let clamp = |min_v, max_v, v| {
            use std::cmp::{max, min};
            min(max_v,max(min_v,v))
        };
        self.instructions_per_second = 
            clamp(MIN_INS_PER_SECOND, MAX_INS_PER_SECOND, new_ips);
    }
    pub fn get_st_active(&self) -> bool {
        self.st != 0
    }
}

#[allow(dead_code)]
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
    {   //test Dxyn - DRW Vx, Vy, nibble
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x03, 0x61, 0x03, 0xA4, 0x00, 0xD0, 0x12, 0xD0, 0x12]);
        cpu.memory.memset(0x400, 
              &vec![0x80, 0x80]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.reg[0xf] != 0 {
            panic!("VF was set when it shouldn't have been");
        }
        if  cpu.memory.get_cell(3,3) != true ||
            cpu.memory.get_cell(3,4) != true {
            panic!("Did not draw properly: \n{:#?}", cpu.memory);
        }

        attempt(cpu.exec_instruction());
        if cpu.reg[0xf] == 0 {
            panic!("VF wasn't set when it should have been");
        }
        if  cpu.memory.get_cell(3,3) != false ||
            cpu.memory.get_cell(3,4) != false {
            panic!("Did not flip cells when drawing: \n{:#?}", cpu.memory);
        }
    }
    {   //test 00E0 - CLS
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x03, 0x61, 0x03, 0xA4, 0x00, 0xD0, 0x12, 0x00, 0xe0]);
        cpu.memory.memset(0x400, 
              &vec![0x80, 0x80]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if  cpu.memory.get_cell(3,3) != false ||
            cpu.memory.get_cell(3,4) != false {
            panic!("Test failed for ins 00E0");
        }
    }
    {   //test Ex9E - SKP Vx
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x61, 0x01, 0xE1, 0x9E, 0xE1, 0x9E]);
        attempt(cpu.exec_instruction());
        cpu.keys[1]=false;
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x204 {
            panic!("Test failed for ins Ex9E on branch not taken path");
        }
        cpu.keys[1]=true;
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x208 {
            panic!("Test failed for ins Ex9E on branch taken path");
        }
    }
    {   //test ExA1 - SKNP Vx
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x61, 0x01, 0xE1, 0xA1, 0xE1, 0xA1]);
        attempt(cpu.exec_instruction());
        cpu.keys[1]=true;
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x204 {
            panic!("Test failed for ins ExA1 on branch not taken path");
        }
        cpu.keys[1]=false;
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x208 {
            panic!("Test failed for ins ExA1 on branch taken path");
        }
    }
    {   //test Fx07 - LD Vx, DT
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0xF0, 0x07]);
        cpu.dt=4;
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x04 {
            panic!("Test failed for ins Fx07");
        }
    }
    {   //test Fx0A - LD Vx, K
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0xF0, 0x0A]);
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x200 {
            panic!("Test failed for ins Fx07: pc should not have incremented");
        }
        attempt(cpu.exec_instruction());
        if cpu.pc != 0x200 {
            panic!("Test failed for ins Fx07: pc should not have incremented");
        }
        cpu.keydown(0x04);
        attempt(cpu.exec_instruction());
        if cpu.reg[0] != 0x04 {
            panic!("Test failed for ins Fx07: did not store pressed key");
        }
        if cpu.pc == 0x200 {
            panic!("Test failed for ins Fx07: pc should have incremented");
        }
    }
    {   //test Fx15 - LD DT, Vx
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x04, 0xF0, 0x15]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.dt != 0x04 {
            panic!("Test failed for ins Fx15");
        }
    }
    {   //test Fx18 - LD ST, Vx
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x60, 0x04, 0xF0, 0x18]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if cpu.st != 0x04 {
            panic!("Test failed for ins Fx18");
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
    {   //test Fx29 - LD F, Vx
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
    {   //test Fx33 - LD B, Vx
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
    {   //test Fx55 - LD [I], Vx
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
    {   //test Fx65 - LD Vx, [I]
        cpu.reset();
        cpu.memory.memset(0x200, 
              &vec![0x63, 0x0f, 0xf3, 0x29, 0xF4, 0x65]);
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        attempt(cpu.exec_instruction());
        if  cpu.reg[0] != 0xf0 ||
            cpu.reg[1] != 0x80 ||
            cpu.reg[2] != 0xf0 ||
            cpu.reg[3] != 0x80 ||
            cpu.reg[4] != 0x80 {
            panic!("Test failed for ins Fx65");
        }
    }
}
