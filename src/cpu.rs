// A chip8 emulator
use core::fmt;
use rand::Rng;
use std::fs::File;
use std::io;
use std::io::Read;

#[derive(Debug)]
pub struct Chip8 {
    opcode: u16, // op pointer
    // System Memory Map:
    // 0x000-0x1FF - The Chip8 Interpreter (contains a font set)
    // 0x050-0x0A0 - Contains the font set
    // 0x200-0xFFF - Program ROM and work RAM 4k memory addresses
    memory: [u8; 4096],
    v: [u8; 16],             // CPU registers
    i: u16,                  // index register
    pc: u16,                 // program counter
    pub gfx: [[u8; 64]; 32], // gfx: the screen
    // timers (60hz) when set >0 they will count down to 0
    delay_timer: u8,
    sound_timer: u8,  // system buzzer makes sound when sound timer reaches 0
    stack: [u16; 16], // the stack memory addresses
    sp: u8,           // the stack pointer
    keys: [u8; 16],   // the 16 keys that can control the system
    screen_updated: bool,
}

// Formatting for printing a Chip8 used to debug state.
impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "------------------------
Chip8:
opcode: 0x{:02x}
index register: {}
program counter: {}
stack pointer: {}
------------------------
",
            self.opcode, self.i, self.pc, self.sp,
        )
    }
}

// Each line represents a character and is annotated accordingly.
const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl Default for Chip8 {
    // Initilizes all components of the system and loads the fontset
    // into memory.
    fn default() -> Self {
        let mut c8 = Chip8 {
            opcode: 0,
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            gfx: [[0; 64]; 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
            screen_updated: false,
        };

        for i in 0..80 {
            c8.memory[i] = CHIP8_FONTSET[i];
        }

        c8
    }
}

impl Chip8 {
    // Loads the game from the filesystem into memory
    pub fn load_game(&mut self, file_name: &str) -> Result<(), io::Error> {
        // TODO: check file_name for .ch8 ending
        let mut file = match File::open(file_name) {
            Ok(val) => val,
            Err(e) => return Err(e),
        };

        // 0x200 -> 0xFFF
        // 512 -> 4096 = 3584 bytes
        // read the file into this buffer

        // read in the file as a byte vector
        let mut buffer: [u8; 3584] = [0; 3584];
        let _size = match file.read(&mut buffer) {
            Ok(val) => val,
            Err(e) => return Err(e),
        };

        // load the game into memory
        for i in 0..3584 {
            //print!("0x{:02x}  ", buffer[i]);
            self.memory[0x200 + i] = buffer[i];
        }

        Ok(())
    }

    // This is the main cycle that consists of three phases
    // Fetch, Decode, and Execute
    // is also responsible for updating timers!!
    pub fn emulate_cycle(&mut self){
        // Fetch opcode
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[(self.pc + 1) as usize] as u16;

        // Decode opcode is done with the match
        // Execute opcode
        match self.execute_opcode() {
            Ok(()) => {
                // update timers
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }
                if self.sound_timer > 0 {
                    self.sound_timer -= 1;
                }
            },
            Err(e) => panic!("{}", e),
        }

    }

    // use the vf register to check whether the scene has been updated
    // by drawing a new sprite to the gfx
    pub fn draw_flag(&mut self) -> bool {
        // if the draw flag is set, reset it and return true
        if self.screen_updated {
            self.screen_updated = false;
            return true;
        }

        false
    }

    // Sets the keys for the
    pub fn set_keys(&mut self, keys: &[u8; 16]) {
        self.keys.copy_from_slice(keys);
    }

    // finds the appropriate opcode function to call
    // and executes it.
    // updates the program counter
    fn execute_opcode(&mut self) -> Result<(), &str> {
        // pull out the last three parts of the opcode into an array
        // this will be passed to the opcode functions to reduce
        // code duplication
        let n: (u8, u8, u8) = (
            ((self.opcode & 0x0F00) >> 8) as u8,
            ((self.opcode & 0x00F0) >> 4) as u8,
            (self.opcode & 0x000F) as u8,
        );
        let x: u8 = n.0;
        let y: u8 = n.1;
        let n: u8 = n.2;
        let nn: u8 = (self.opcode & 0x00FF) as u8;
        let nnn: u16 = self.opcode & 0x0FFF;

        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x00FF {
                // clear screen
                0x00E0 => {
                    self.gfx = [[0; 64]; 32];
                    self.screen_updated = true;
                    self.pc += 2;
                    Ok(())
                }
                0x00EE => Ok(self.return_subroutine()),
                _ => panic!("opcode decoded an unsupported code: 0x{:02x}!", self.opcode),
            },
            // jump to address NNN
            0x1000 => {
                let new_addr = self.opcode & 0x0FFF;
                self.pc = new_addr;
                Ok(())
            }
            0x2000 => Ok(self.call_subroutine_at_nnn(&nnn)),
            0x3000 => Ok(self.skip_if_vx_equals_nn(&x, &nn)),
            0x4000 => Ok(self.skip_if_vx_not_equal_nn(&x, &nn)),
            0x5000 => Ok(self.skip_if_vx_equals_vy(&x, &y)),
            0x6000 => Ok(self.vx_equals_nn(&x, &nn)),
            0x7000 => Ok(self.vx_plus_equals_nn(&x, &nn)),
            0x8000 => match self.opcode & 0x000f {
                0x0000 => Ok(self.vx_assign_vy(&x, &y)),
                0x0001 => Ok(self.vx_assign_or_vy(&x, &y)),
                0x0002 => Ok(self.vx_assign_and_vy(&x, &y)),
                0x0003 => Ok(self.vx_assign_xor_vy(&x, &y)),
                0x0004 => Ok(self.vx_assign_plus_vy(&x, &y)),
                0x0005 => Ok(self.vx_assign_minus_vy(&x, &y)),
                0x0006 => Ok(self.vx_assign_rshift(&x)),
                0x0007 => Ok(self.vx_assign_vy_minus_vx(&x, &y)),
                0x000e => Ok(self.vx_assign_lshift(&x)),
                _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
            },
            0x9000 => Ok(self.skip_if_vx_not_equal_vy()),
            // set i to addr nnn
            0xa000 => {
                self.i = nnn;
                self.pc += 2;
                Ok(())
            }
            // pc = v0 + nnn
            0xb000 => Ok(self.pc = self.v[0] as u16 + nnn),
            0xc000 => Ok(self.vx_equals_rand(&x, &nn)),
            0xd000 => Ok(self.draw(&x, &y, &n)),
            0xe000 => match self.opcode & 0x000f {
                0x000e => Ok(self.skip_if_key_pressed(&x)),
                0x0001 => Ok(self.skip_if_key_not_pressed(&x)),
                _ => panic!("opcode decoded an unsupported code: 0x{:02x}!", self.opcode),
            },
            0xf000 => match self.opcode & 0x00ff {
                0x0007 => Ok(self.vx_assign_delay(&x)),
                0x000a => Ok(self.vx_assign_key(&x)),
                0x0015 => Ok(self.set_delay_timer(&x)),
                0x0018 => Ok(self.set_sound_timer(&x)),
                0x001e => Ok(self.index_assign_plus_vx(&x)),
                0x0029 => Ok(self.index_assign_sprite(&x)),
                0x0033 => Ok(self.set_bcd(&x)),
                0x0055 => Ok(self.reg_dump(&x)),
                0x0065 => Ok(self.reg_load(&x)),
                _ => panic!("opcode decoded an unsupported code: 0x{:02x}!", self.opcode),
            },
            _ => panic!("opcode decoded an unsupported code: 0x{:02x}!", self.opcode),
        }
    }

    // returns from the subroutine
    #[inline]
    fn return_subroutine(&mut self) {
        self.pc = self.stack[self.sp as usize];

        // Make sure stack pointer stays in bounds
        if self.sp > 0 {
            self.sp -= 1;
        }
    }

    // call the subroutine at the memory address nnn in opcode
    #[inline]
    fn call_subroutine_at_nnn(&mut self, nnn: &u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc + 2;
        self.pc = *nnn;
    }

    // skip the next instruction if Vx == NN
    #[inline]
    fn skip_if_vx_equals_nn(&mut self, x: &u8, nn: &u8) {
        if self.v[*x as usize] == *nn {
            self.pc += 2;
        }
        self.pc += 2;
    }

    // skip the next instruction if Vx != NN
    #[inline]
    fn skip_if_vx_not_equal_nn(&mut self, x: &u8, nn: &u8) {
        if self.v[*x as usize] != *nn {
            self.pc += 2;
        }
        self.pc += 2;
    }

    // skip the next instruction if Vx != Vy
    #[inline]
    fn skip_if_vx_equals_vy(&mut self, x: &u8, y: &u8) {
        if self.v[*x as usize] == self.v[*y as usize] {
            self.pc += 2;
        }

        self.pc += 2;
    }

    // sets Vx equal to NN
    #[inline]
    fn vx_equals_nn(&mut self, x: &u8, nn: &u8) {
        self.v[*x as usize] = *nn;
        self.pc += 2;
    }

    // add vx and nn and assign to vx
    // 0x7xnn
    #[inline]
    fn vx_plus_equals_nn(&mut self, x: &u8, nn: &u8) {
        if self.v[*x as usize].checked_add(*nn).is_none() {
            self.v[*x as usize] = ((self.v[*x as usize] as u16 + *nn as u16) % 256) as u8;
        } else {
            self.v[*x as usize] += *nn;
        }

        self.pc += 2;
    }

    // vx = vy
    // 0x8xy0
    #[inline]
    fn vx_assign_vy(&mut self, x: &u8, y: &u8) {
        self.v[*x as usize] = self.v[*y as usize];
        self.pc += 2;
    }

    // vx |= vy
    // 0x8xy1
    #[inline]
    fn vx_assign_or_vy(&mut self, x: &u8, y: &u8) {
        self.v[*x as usize] |= self.v[*y as usize];
        self.pc += 2;
    }

    // vx &= vy
    #[inline]
    fn vx_assign_and_vy(&mut self, x: &u8, y: &u8) {
        self.v[*x as usize] &= self.v[*y as usize];
        self.pc += 2;
    }

    // vx ^= vy
    #[inline]
    fn vx_assign_xor_vy(&mut self, x: &u8, y: &u8) {
        self.v[*x as usize] ^= self.v[*y as usize];
        self.pc += 2;
    }

    // vx += vy
    #[inline]
    fn vx_assign_plus_vy(&mut self, x: &u8, y: &u8) {
        if self.v[*x as usize] > (255 - self.v[*y as usize]) {
            self.v[0xF] = 1; // carry
            self.v[*x as usize] =
                ((self.v[*y as usize] as u16 + self.v[*x as usize] as u16) - 256) as u8;
        } else {
            self.v[0xF] = 0;
            self.v[*x as usize] += self.v[*y as usize];
        }

        self.pc += 2;
    }

    // vx -= vy
    #[inline]
    fn vx_assign_minus_vy(&mut self, x: &u8, y: &u8) {
        let vx = self.v[*x as usize];
        let vy = self.v[*y as usize];
        if vx < vy {
            self.v[0xF] = 0;
            self.v[*x as usize] =
                ((self.v[*x as usize] as u16 + 256) - self.v[*y as usize] as u16) as u8;
        } else {
            self.v[0xF] = 1;
            self.v[*x as usize] -= self.v[*y as usize];
        }

        //panic!("{} - {} = {}", vx, vy, self.v[*x as usize]);

        self.pc += 2;
    }

    // vx >>= 1
    #[inline]
    fn vx_assign_rshift(&mut self, x: &u8) {
        self.v[0xF] = self.v[*x as usize] & 1;

        self.v[*x as usize] >>= 1;
        self.pc += 2;
    }

    // vx = vy - vx
    #[inline]
    fn vx_assign_vy_minus_vx(&mut self, x: &u8, y: &u8) {
        let vx = self.v[*x as usize];
        let vy = self.v[*y as usize];
        if vy < vx {
            self.v[0xF] = 0;
            self.v[*x as usize] =
                ((self.v[*y as usize] as u16 + 256) - self.v[*x as usize] as u16) as u8;
        } else {
            self.v[0xF] = 1;
            self.v[*x as usize] = self.v[*y as usize] - self.v[*x as usize];
        }

        //panic!("{} - {} = {}", vx, vy, self.v[*x as usize]);

        self.pc += 2;
    }

    // vx <<= 1
    #[inline]
    fn vx_assign_lshift(&mut self, x: &u8) {
        self.v[0xF] = self.v[*x as usize] >> 7;
        self.v[*x as usize] <<= 1;
        self.pc += 2;
    }

    // if (vx != vy)
    #[inline]
    fn skip_if_vx_not_equal_vy(&mut self) {
        if self.v[((self.opcode & 0x0F00) >> 8) as usize]
            != self.v[((self.opcode & 0x00F0) >> 4) as usize]
        {
            self.pc += 2;
        }
        self.pc += 2;
    }

    // vx = rand() & nn
    #[inline]
    fn vx_equals_rand(&mut self, x: &u8, nn: &u8) {
        let r = rand::thread_rng().gen_range(0..=255);
        self.v[*x as usize] = r & *nn;
        self.pc += 2;
    }

    // draw(vx, vy, n)
    // draw sprite at I for n rows
    #[inline]
    fn draw(&mut self, x: &u8, y: &u8, n: &u8) {
        // pull out the three arguments
        // make x and y cords stay on screen by bitwise-& width or height

        // set the overflow register to 0
        // we will update this to 1 if the sprite goes off screen
        self.v[0xF] = 0;

        // Update gfx
        for row in 0..*n {
            // dont go off the screen vertically
            let vy = (self.v[*y as usize] as u16 + row as u16) % 32;
            // grab the sprite from I!
            let sprite = self.memory[(self.i + row as u16) as usize];

            // Update each pixel
            for pixel in 0..8 {
                let vx = (self.v[*x as usize] as u16 + pixel as u16) % 64;
                // dont go off the screen horizontally
                let color = (sprite >> (7 - pixel)) & 1;
                self.v[0xF] |= color & self.gfx[vy as usize][vx as usize];
                self.gfx[vy as usize][vx as usize] ^= color;
            }
        }

        // set the draw flags to true so this gets rendered!
        self.screen_updated = true;
        self.pc += 2;
    }

    // if (key() == vx)
    #[inline]
    fn skip_if_key_pressed(&mut self, x: &u8) {
        if self.keys[self.v[*x as usize] as usize] != 0 {
            self.pc += 2;
        }
        self.pc += 2;
    }

    // if (key() != vx)
    #[inline]
    fn skip_if_key_not_pressed(&mut self, x: &u8) {
        if self.keys[self.v[*x as usize] as usize] == 0 {
            self.pc += 2;
        }
        self.pc += 2;
    }

    // vx = get_delay()
    #[inline]
    fn vx_assign_delay(&mut self, x: &u8) {
        self.v[*x as usize] = self.delay_timer;
        self.pc += 2;
    }

    // vx = get_key()
    #[inline]
    fn vx_assign_key(&mut self, x: &u8) {
        if self.keys.contains(&255) {
            for (i, key) in self.keys.iter().enumerate() {
                if *key != 0 as u8 {
                    self.v[*x as usize] = i as u8;
                    break;
                }
            }
            self.pc += 2;
        }
    }

    // set_delay(vx)
    #[inline]
    fn set_delay_timer(&mut self, x: &u8) {
        self.delay_timer = self.v[*x as usize];
        self.pc += 2;
    }

    // set sound timer
    #[inline]
    fn set_sound_timer(&mut self, x: &u8) {
        self.sound_timer = self.v[*x as usize];
        self.pc += 2;
    }

    #[inline]
    fn index_assign_plus_vx(&mut self, x: &u8) {
        self.i += self.v[*x as usize] as u16;
        self.pc += 2;
    }

    #[inline]
    fn index_assign_sprite(&mut self, x: &u8) {
        self.i = (self.v[*x as usize] & 0xF) as u16 * 5;

        self.pc += 2;
    }

    #[inline]
    fn set_bcd(&mut self, x: &u8) {
        self.memory[self.i as usize] = self.v[*x as usize] / 100;
        self.memory[self.i as usize + 1] = (self.v[*x as usize] % 100) / 10;
        self.memory[self.i as usize + 2] = self.v[*x as usize] % 10;

        self.pc += 2;
    }

    #[inline]
    fn reg_dump(&mut self, x: &u8) {
        for reg in 0..=*x {
            self.memory[self.i as usize + reg as usize] = self.v[reg as usize];
        }

        self.pc += 2;
    }

    #[inline]
    fn reg_load(&mut self, x: &u8) {
        for reg in 0..=*x {
            self.v[reg as usize] = self.memory[self.i as usize + reg as usize];
        }

        self.pc += 2;
    }
}

#[cfg(test)]
mod tests {
    use crate::Chip8;

    #[test]
    fn return_subroutine_with_empty_stack() {
        let mut cpu = Chip8::default();
        cpu.return_subroutine();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn return_subroutine_with_value() {
        let mut cpu = Chip8::default();
        cpu.stack[0] = 0x300;
        cpu.return_subroutine();
        assert_eq!(cpu.pc, 0x300);
    }

    #[test]
    fn return_subroutine_with_maxvalue() {
        let mut cpu = Chip8::default();
        cpu.stack[0] = 0xFFF;
        cpu.return_subroutine();
        assert_eq!(cpu.pc, 0xFFF);
    }

    #[test]
    fn return_subroutine_sp_not_zero() {
        let mut cpu = Chip8::default();
        cpu.stack[8] = 0x500;
        cpu.sp = 8;
        cpu.return_subroutine();
        assert_eq!(cpu.pc, 0x500);
    }

    #[test]
    fn lshift_sets_msb() {
        let mut cpu = Chip8::default();
        cpu.v[2] = 0b10101010;
        cpu.vx_assign_lshift(&2);
        assert_eq!(cpu.v[0xF], 1);
        assert_eq!(cpu.v[2], 0b01010100);
    }

    #[test]
    fn set_bcd_correctly() {
        let mut cpu = Chip8::default();
        cpu.v[4] = 23;
        cpu.i = 0x30;

        cpu.set_bcd(&4);
        assert_eq!(cpu.memory[0x30], 0);
        assert_eq!(cpu.memory[0x31], 2);
        assert_eq!(cpu.memory[0x32], 3);
    }

    #[test]
    fn read_bcd_correctly() {
        let mut cpu = Chip8::default();
        cpu.v[4] = 123;
        cpu.i = 0x30;

        cpu.set_bcd(&4);
        assert_eq!(cpu.memory[0x30], 1);
        assert_eq!(cpu.memory[0x31], 2);
        assert_eq!(cpu.memory[0x32], 3);

        cpu.reg_load(&2);
        assert_eq!(cpu.v[0], 1);
        assert_eq!(cpu.v[1], 2);
        assert_eq!(cpu.v[2], 3);
    }
}
