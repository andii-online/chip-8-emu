// A chip8 emulator
pub mod chip8 {
    use core::fmt;
    use rand::Rng;
    use sdl2::libc::processor_cpu_load_info_t;
    use std::fs::File;
    use std::io::Read;

    #[derive(Debug)]
    pub struct Chip8 {
        opcode: u16, // op pointer
        // System Memory Map:
        // 0x000-0x1FF - The Chip8 Interpreter (contains a font set)
        // 0x050-0x0A0 - Contains the font set
        // 0x200-0xFFF - Program ROM and work RAM
        // 4k memory addresses
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

    impl Chip8 {
        // Initilizes all components of the system and loads the fontset
        // into memory.
        pub fn new() -> Chip8 {
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

        // Loads the game from the filesystem into memory
        pub fn load_game(&mut self, file_name: &str) {
            // TODO: check file_name for .ch8 ending

            let mut file = match File::open(file_name) {
                Ok(val) => val,
                Err(e) => panic!("Error loading game from file: {}", e),
            };

            // 0x200 -> 0xFFF
            // 512 -> 4096 = 3584 bytes
            // read the file into this buffer

            // read in the file as a byte vector
            let mut buffer: [u8; 3584] = [0; 3584];
            let _size = match file.read(&mut buffer) {
                Ok(val) => val,
                Err(e) => panic!("error reading file: {}", e),
            };

            // load the game into memory
            for i in 0..3584 {
                //print!("0x{:02x}  ", buffer[i]);
                self.memory[0x200 + i] = buffer[i];
            }
        }

        // This is the main cycle that consists of three phases
        // Fetch, Decode, and Execute
        // is also responsible for updating timers!!
        pub fn emulate_cycle(&mut self) {
            //println!("{}", self);

            // Fetch opcode
            self.opcode = (self.memory[self.pc as usize] as u16) << 8
                | self.memory[(self.pc + 1) as usize] as u16;

            // Decode opcode is done with the match
            // Execute opcode
            self.execute_opcode();

            // update timers
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

        // todo
        pub fn set_keys(&self) {}

        // finds the appropriate opcode function to call
        // and executes it.
        // updates the program counter
        fn execute_opcode(&mut self) {
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

            for (i, mem) in self.v.iter().enumerate() {
                print!("|v{}: {} ", i, mem);
                if i % 4 == 3 {
                    println!("|");
                }
            }

            println!("opcode: 0x{:02x}", self.opcode);
            println!("x:{} y:{} n:{}", x, y, n);
            println!();

            match self.opcode & 0xF000 {
                0x0000 => match self.opcode & 0x00F {
                    // clear screen
                    0x0000 => {
                        self.gfx = [[0; 64]; 32];
                        self.screen_updated = true;
                        self.pc += 2;
                    }
                    0x000E => self.return_subroutine(),
                    _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
                },
                // jump to address NNN
                0x1000 => {
                    let new_addr = self.opcode & 0x0FFF;
                    self.pc = new_addr;
                }
                0x2000 => self.call_subroutine_at_nnn(&nnn),
                0x3000 => self.skip_if_vx_equals_nn(&x, &nn),
                0x4000 => self.skip_if_vx_not_equal_nn(&x, &nn),
                0x5000 => self.skip_if_vx_equals_vy(&x, &y),
                0x6000 => self.vx_equals_nn(&x, &nn),
                0x7000 => self.vx_plus_equals_nn(&x, &nn),
                0x8000 => match self.opcode & 0x000f {
                    0x0000 => self.vx_assign_vy(&x, &y),
                    0x0001 => self.vx_assign_or_vy(&x, &y),
                    0x0002 => self.vx_assign_and_vy(&x, &y),
                    0x0003 => self.vx_assign_xor_vy(&x, &y),
                    0x0004 => self.vx_assign_plus_vy(&x, &y),
                    0x0005 => self.vx_assign_minus_vy(&x, &y),
                    0x0006 => self.vx_assign_rshift(&x),
                    0x0007 => self.vx_assign_vy_minus_vx(),
                    0x000e => self.vx_assign_lshift(),
                    _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
                },
                0x9000 => self.skip_if_vx_not_equal_vy(),
                // set i to addr nnn
                0xa000 => {
                    self.i = nnn;
                    self.pc += 2;
                }
                // pc = v0 + nnn
                0xb000 => self.pc = self.v[0] as u16 + nnn,
                0xc000 => self.vx_equals_rand(),
                0xd000 => self.draw(&x, &y, &n),
                0xe000 => match self.opcode & 0x000f {
                    0x000e => self.skip_if_key_pressed(),
                    0x0001 => self.skip_if_key_not_pressed(),
                    _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
                },
                0xf000 => match self.opcode & 0x00ff {
                    0x0007 => self.vx_assign_delay(),
                    0x000a => self.vx_assign_key(),
                    0x0015 => self.set_delay_timer(),
                    0x0018 => self.set_sound_timer(),
                    0x001e => self.index_assign_plus_vx(),
                    0x0029 => self.index_assign_sprite(),
                    0x0033 => self.set_bcd(),
                    0x0055 => self.reg_dump(),
                    0x0065 => self.reg_load(),
                    _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
                },
                _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
            }
        }

        // returns from the subroutine
        fn return_subroutine(&mut self) {
            self.pc = self.stack[self.sp as usize];
            self.sp -= 1;
        }

        // call the subroutine at the memory address nnn in opcode
        fn call_subroutine_at_nnn(&mut self, nnn: &u16) {
            self.sp += 1;
            self.stack[self.sp as usize] = self.pc;
            self.pc = *nnn;
        }

        // skip the next instruction if Vx == NN
        fn skip_if_vx_equals_nn(&mut self, x: &u8, nn: &u8) {
            if *x == *nn {
                self.pc += 2;
            }
            self.pc += 2;
        }

        // skip the next instruction if Vx != NN
        fn skip_if_vx_not_equal_nn(&mut self, x: &u8, nn: &u8) {
            if self.v[*x as usize] != *nn {
                self.pc += 2;
            }
            self.pc += 2;
        }

        // skip the next instruction if Vx != Vy
        fn skip_if_vx_equals_vy(&mut self, x: &u8, y: &u8) {
            if self.v[*x as usize] == self.v[*y as usize] {
                self.pc += 2;
            }

            self.pc += 2;
        }

        // sets Vx equal to NN
        fn vx_equals_nn(&mut self, x: &u8, nn: &u8) {
            //println!("set v{} to {}", *x, *nn);
            self.v[*x as usize] = *nn;
            self.pc += 2;
        }

        // add vx and nn and assign to vx
        // 0x7xnn
        fn vx_plus_equals_nn(&mut self, x: &u8, nn: &u8) {
            if self.v[*x as usize].checked_add(*nn).is_none() {
                self.v[*x as usize] += 255 - *nn;
                self.v[0xF] = 255;
            } else {
                self.v[*x as usize] += *nn;
            }

            self.pc += 2;
        }

        // vx = vy
        // 0x8xy0
        fn vx_assign_vy(&mut self, x: &u8, y: &u8) {
            self.v[*x as usize] = self.v[*y as usize];
            self.pc += 2;
        }

        // vx |= vy
        // 0x8xy1
        fn vx_assign_or_vy(&mut self, x: &u8, y: &u8) {
            self.v[*x as usize] |= self.v[*y as usize];
            self.pc += 2;
        }

        // vx &= vy
        fn vx_assign_and_vy(&mut self, x: &u8, y: &u8) {
            self.v[*x as usize] &= self.v[*y as usize];
            self.pc += 2;
        }

        // vx ^= vy
        fn vx_assign_xor_vy(&mut self, x: &u8, y: &u8) {
            self.v[*x as usize] ^= self.v[*y as usize];
            self.pc += 2;
        }

        // vx += vy
        fn vx_assign_plus_vy(&mut self, x: &u8, y: &u8) {
            if self.v[*x as usize] > (0xFF - self.v[*y as usize]) {
                self.v[0xF] = 1; // carry
            } else {
                self.v[0xF] = 0;
            }
            self.v[*x as usize] += self.v[*y as usize];
            self.pc += 2;
        }

        // vx -= vy
        fn vx_assign_minus_vy(&mut self, x: &u8, y: &u8) {
            let vx = &self.v[*x as usize];
            let vy = &self.v[*y as usize];
            if vx - vy < 0xFF {
                self.v[0xF] = 0;
            } else {
                self.v[0xF] = 1;
            }

            self.v[*x as usize] -= self.v[*y as usize];
            self.pc += 2;
        }

        // vx >>= 1
        fn vx_assign_rshift(&mut self, x: &u8) {
            if (x & 0b1) == 1 {
                self.v[0xF] = 1;
            } else {
                self.v[0xF] = 0;
            }

            self.v[*x as usize] /= 2;
            self.pc += 2;
        }

        // vx = vy - vx
        fn vx_assign_vy_minus_vx(&mut self) {
            if self.v[((self.opcode & 0x00F0) >> 4) as usize]
                < self.v[((self.opcode & 0x0F00) >> 8) as usize]
            {
                self.v[0xF] = 0;
            } else {
                self.v[0xF] = 1;
            }

            self.v[((self.opcode & 0x0F00) >> 8) as usize] = self.v
                [((self.opcode & 0x00F0) >> 4) as usize]
                - self.v[((self.opcode & 0x0F00) >> 8) as usize];
            self.pc += 2;
        }

        // vx <<= 1
        fn vx_assign_lshift(&mut self) {
            let x = (self.opcode & 0x0F00) >> 8;
            if (x & 0b10000000) == 1 {
                self.v[0xF] = 1;
            } else {
                self.v[0xF] = 0;
            }
            self.v[((self.opcode & 0x0F00) >> 8) as usize] *= 2;
            self.pc += 2;
        }

        // if (vx != vy)
        fn skip_if_vx_not_equal_vy(&mut self) {
            if self.v[((self.opcode & 0x0F00) >> 8) as usize]
                != self.v[((self.opcode & 0x00F0) >> 4) as usize]
            {
                self.pc += 2;
            }
            self.pc += 2;
        }

        // vx = rand() & nn
        fn vx_equals_rand(&mut self) {
            let r = rand::thread_rng().gen_range(0..=255);
            self.v[((self.opcode & 0x0F00) >> 8) as usize] =
                r & (self.opcode & 0x00FF).to_be_bytes()[1];
        }

        // draw(vx, vy, n)
        // draw sprite at I for n rows
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
        fn skip_if_key_pressed(&mut self) {
            if self.v[((self.opcode & 0x0F00) >> 8) as usize]
                == self.keys[((self.opcode & 0x00F0) >> 4) as usize]
            {
                self.pc += 2;
            }
            self.pc += 2;
        }

        // if (key() != vx)
        fn skip_if_key_not_pressed(&mut self) {
            if self.v[((self.opcode & 0x0F00) >> 8) as usize]
                != self.keys[((self.opcode & 0x00F0) >> 4) as usize]
            {
                self.pc += 2;
            }
            self.pc += 2;
        }

        // vx = get_delay()
        fn vx_assign_delay(&mut self) {}

        // vx = get_key()
        fn vx_assign_key(&mut self) {}

        // set_delay(vx)
        fn set_delay_timer(&mut self) {}

        // set sound timer
        fn set_sound_timer(&mut self) {}

        fn index_assign_plus_vx(&mut self) {}

        fn index_assign_sprite(&mut self) {}

        fn set_bcd(&mut self) {}

        fn reg_dump(&mut self) {}

        fn reg_load(&mut self) {}
    }
}
