use mem::Mem;

#[derive(Debug)]
pub struct Cpu {
    pc : u16,
    sp : u8,
    reg : [u8; 16],
    stack : [u16; 16],
    reg_I : u16,
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
            reg_I : 0,
            //ins : 0,
            memory : Default::default()
        };
        ret_val
    }
    pub fn exec_instruction(&mut self) {
        let ins = self.get_next_instruction();

        self.pc += 2;
    }
    fn get_next_instruction(&mut self) -> u16 {
        self.memory.read_u16(self.pc as usize)
    }
}

#[test]
fn test_exec_instruction(){
    let mut cpu = Cpu::new();

}
