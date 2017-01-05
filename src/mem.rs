use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::path;

pub struct Mem {
    mem : [u8; 0x1000]
}

impl Mem {
    pub fn load_rom(&mut self, file_name: &path::Path) -> Result<(), &str> {
        Ok(())
    }
    pub fn read_u16(&mut self, addr: usize) -> u16 {
        let mut foo = &self.mem[addr..addr+2];
        foo.read_u16::<BigEndian>().unwrap()
    }
    pub fn read_u8(&mut self, addr: usize) -> u8 {
        self.mem[addr]
    }
    pub fn write_u16(&mut self, addr: usize, val: u16) {
        let mut foo = &mut self.mem[addr..addr+2];
        foo.write_u16::<BigEndian>(val).unwrap()
    }
    pub fn write_u8(&mut self, addr: usize, val: u8) {
        self.mem[addr]=val;
    }
    pub fn memset(&mut self, addr: usize, data: &Vec<u8>){
        for (i, v) in data.iter().enumerate() {
            self.mem[addr+i]=*v;
        }
    }
}

impl Default for Mem {
    fn default() -> Mem {
        Mem {
            mem : [0u8; 0x1000]
        }
    }
}

impl fmt::Debug for Mem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (addr, val) in self.mem.iter().enumerate() {
            if (addr % 16) == 0 {
                try!(write!(f, "\n    0x{:03x}:", addr));
            }
            if (addr % 2) == 0 {
                try!(write!(f, " "));
            }
            try!(write!(f, "{:02x}", val));
        }
        try!(write!(f, "\n"));
        Ok(())
    }
}

#[test]
fn test_memory_access(){
    let mut mem :Mem = Default::default();
    let magic_value: u16 = 517;
    mem.write_u16(0x200,magic_value);
    if mem.read_u16(0x200) != magic_value {
        panic!("Error writing/reading 16bit value to/from Mem");
    }
    mem.write_u8(0x200, 40);
    if mem.read_u8(0x200) != 40 {
        panic!("Error writing/reading 8bit value to/from Mem");
    }
    if mem.read_u8(0x201) != 5 {
        panic!("Data written in little endian?");
    }
}
