use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::Read;

pub const MEM_SIZE : usize =   0x1000;
pub const SCREEN_WIDTH : usize =   64;
pub const SCREEN_HEIGHT: usize =   32;
pub const SPRITE_DATA_OFFSET: u16 = 0;

pub struct Mem {
    mem : [u8; MEM_SIZE],
    vmem: [[bool; 256]; 256]
}

impl Mem {
    //Manipulate large amounts of memory directly
    pub fn load_rom(&mut self, file_name: &String) -> Result<(), io::Error> {

        let file_name= Path::new(file_name);
        let mut f = try!(File::open(file_name));
        let mut file_vec: Vec<u8> = Vec::new();
        let bytes_read = try!(f.read_to_end(&mut file_vec));
        println!("{} bytes read", bytes_read);
        if bytes_read > (MEM_SIZE - 0x200) {
            let e = io::Error::new(io::ErrorKind::Other, "Rom file too large");
            return Err(e);
        }
        self.memset(0x200, &file_vec);
        Ok(())
        
    }
    pub fn memset(&mut self, addr: usize, data: &Vec<u8>){
        for (i, v) in data.iter().enumerate() {
            self.mem[addr+i]=*v;
        }
    }
    pub fn get_vec(&self, addr : usize, n: usize) -> Vec<u8> {
        self.mem[addr..addr+n].to_vec()
    }
    pub fn reset(&mut self){
        self.mem = [0u8; MEM_SIZE];
        self.clear_screen();
        self.set_sprite_data();
    }

    //Memory access functions
    pub fn read_u16(&self, addr: usize) -> u16 {
        let mut foo = &self.mem[addr..addr+2];
        foo.read_u16::<BigEndian>().unwrap()
    }
    #[allow(dead_code)]
    pub fn read_u8(&mut self, addr: usize) -> u8 {
        self.mem[addr]
    }
    #[allow(dead_code)]
    pub fn write_u16(&mut self, addr: usize, val: u16) {
        let mut foo = &mut self.mem[addr..addr+2];
        foo.write_u16::<BigEndian>(val).unwrap()
    }
    pub fn write_u8(&mut self, addr: usize, val: u8) {
        self.mem[addr]=val;
    }

    //vmem functions
    pub fn clear_screen(&mut self){
        for x in 0..256 {
            for y in 0..256 {
                self.vmem[x as usize][y as usize] = false;
            }
        }
    }
    pub fn get_cell(&self, x: u8, y: u8) -> bool {
        self.vmem[x as usize][y as usize]
    }
    pub fn draw_sprite(&mut self, addr: u16, x: u8, y: u8, n: u8) -> bool {
        let sprite_vec: Vec<u8> = self.get_vec(addr as usize, n as usize);
        let mut ret_val = false;
        for (yi, val) in sprite_vec.iter().enumerate() {

            let mut draw_pixel = |x: usize, y: usize| {
                if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
                    self.vmem[x][y]=!self.vmem[x][y];
                    if !self.vmem[x][y] { 
                        ret_val=true;
                    }
                }
            };

            for xi in 0..8 {
                let flag = (val>>(7-xi)) & 1u8;
                if flag == 1u8 {
                    draw_pixel((x as usize)+xi, (y as usize) + yi);
                }
            }

        }
        ret_val
    }
    fn set_sprite_data(&mut self){
        let sprite_data = 
            vec![0xf0, 0x90, 0x90, 0x90, 0xf0,
                 0x20, 0x60, 0x20, 0x20, 0x70,
                 0xF0, 0x10, 0xF0, 0x80, 0xF0,
                 0xF0, 0x10, 0xF0, 0x10, 0xF0,
                 0x90, 0x90, 0xF0, 0x10, 0x10,
                 0xF0, 0x80, 0xF0, 0x10, 0xF0,
                 0xF0, 0x80, 0xF0, 0x90, 0xF0,
                 0xF0, 0x10, 0x20, 0x40, 0x40,
                 0xF0, 0x90, 0xF0, 0x90, 0xF0,
                 0xF0, 0x90, 0xF0, 0x10, 0xF0,
                 0xF0, 0x90, 0xF0, 0x90, 0x90,
                 0xE0, 0x90, 0xE0, 0x90, 0xE0,
                 0xF0, 0x80, 0x80, 0x80, 0xF0,
                 0xE0, 0x90, 0x90, 0x90, 0xE0,
                 0xF0, 0x80, 0xF0, 0x80, 0xF0,
                 0xF0, 0x80, 0xF0, 0x80, 0x80];
        self.memset(SPRITE_DATA_OFFSET as usize, &sprite_data);
    }
    pub fn get_sprite_addr(sprite_no: u8) -> u16 {
        SPRITE_DATA_OFFSET + ((5*sprite_no) as u16)
    }
}

impl Default for Mem {
    fn default() -> Mem {
        let mut ret_val = Mem {
            mem : [0u8; MEM_SIZE],
            vmem: [[false; 256]; 256]
        };
        ret_val.set_sprite_data();
        ret_val
    }
}

impl fmt::Debug for Mem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "\n    mem: {{"));
        for (addr, val) in self.mem.iter().enumerate() {
            if (addr % 16) == 0 {
                try!(write!(f, "\n        0x{:03x}:", addr));
            }
            if (addr % 2) == 0 {
                try!(write!(f, " "));
            }
            try!(write!(f, "{:02x}", val));
        }
        try!(write!(f, "\n    }},"));
        try!(write!(f, "\n    vmem: {{"));
        for y in 0..SCREEN_HEIGHT {
            try!(write!(f, "\n        "));
            for x in 0..SCREEN_WIDTH {
                match self.vmem[x][y] {
                    true  => try!(write!(f, "X")),
                    false => try!(write!(f, ".")),
                }
            }
        }
        try!(write!(f, "\n    }},"));
        try!(write!(f, "\n"));
        Ok(())
    }
}

#[test]
fn test_memory_access(){
    let mut mem: Mem = Default::default();
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

#[test]
fn test_memset(){
    let mut mem: Mem = Default::default();
    let test_vec = vec![1, 2, 3, 4];
    mem.memset(0x200, &test_vec);
    if  mem.mem[0x200]!=test_vec[0] ||
        mem.mem[0x201]!=test_vec[1] ||
        mem.mem[0x202]!=test_vec[2] ||
        mem.mem[0x203]!=test_vec[3] {
        panic!("Tried to write {:?}, got {:?}", test_vec, mem.mem[0x200..0x204].to_vec());
    }
}

#[test]
fn test_get_vec(){
    let mut mem: Mem = Default::default();
    let test_vec = vec![1, 2, 3, 4];
    mem.memset(0x200, &test_vec);
    let res_vec = mem.get_vec(0x200, 4);
    if res_vec != test_vec {
        panic!("Expected {:?}, got {:?}", test_vec, res_vec);
    }
}
