

// A chip8 emulator
pub mod chip8 {
    pub struct Chip8 {
        opcode: u16, // op pointer
        memory: [u8; 4096], // 4k memory addresses
        v: [u8; 16], // CPU registers
        i: u16, // index register 
        pc: u16, // program counter
        // System Memory Map:
        // 0x000-0x1FF - The Chip8 Interpreter (contains a font set)
        // 0x050-0x0A0 - Contains the font set
        // 0x200-0xFFF - Program ROM and work RAM
        gfx: [u8; 64 * 32],
        // timers (60hz) when set >0 they will count down to 0
        delay_timer: u8, 
        sound_timer: u8, // system buzzer makes sound when sound timer reaches 0
        stack: [u16; 16], // the stack memory addresses
        sp: u8, // the stack pointer
        keys: [u8; 16], // the 16 keys that can control the system
    }

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
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];

    pub fn init() -> Chip8 {
        let mut c8 = Chip8 {
            opcode: 0,
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
        };

        for i in 0..81 {
            c8.memory[i] = CHIP8_FONTSET[i];
        }

        c8
    }

    impl Chip8 {
        pub fn load_game(&self, file_name: &str) {

        }

        pub fn emulate_cycle (&mut self) {
            // Fetch opcode
            self.opcode = (self.memory[self.pc as usize] as u16) << 8 
                | self.memory[(self.pc + 1) as usize] as u16;
            
            // Decode opcode
            match self.opcode & 0xF000 {
                0x0000  =>  match self.opcode & 0x00F {
                                0x0000 => self.gfx = [0; 64 * 32], // clear the display
                                0x000E => self.return_subroutine(), 
                                _ => panic!("opcode decoded an unsupported code: {}!", self.opcode), 
                            }
                0x1000 => self.pc = self.opcode & 0x0FFF, // jump to address NNN 
                0x2000 => self.call_subroutine_at_nnn(), 
                0x3000 => self.skip_if_vx_equals_nn(), 
                0x4000 => self.skip_if_vx_not_equal_nn(), 
                0x5000 => self.skip_if_vx_equals_vy(),
                0x6000 => self.vx_equals_nn(),
                0x7000 => self.vx_plus_equals_nn(), 
                0x8000 =>   match self.opcode & 0x000F {
                                0x0000 => (), // vx = vy
                                0x0001 => (), // vx |= vy
                                0x0002 => (), // vx &= vy
                                0x0003 => (), // vx ^= vy
                                0x0004 => (), // vx += vy
                                0x0005 => (), // vx -= vy
                                0x0006 => (), // vx >>= 1
                                0x0007 => (), // vx = vy - vx
                                0x000E => (), // vx <<= 1
                                _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
                            }
                0x9000 => (), // if (vx != vy)
                0xA000 => (), // I = NNN
                0xB000 => (), // PC = v0 + nnn
                0xC000 => (), // vx = rand() & NN
                0xD000 => (), // draw(vx, vy, n)
                0xE000 =>   match self.opcode & 0x000F {
                                0x000E => (), // if (key() == vx)
                                0x0001 => (), // if (key() != vx)
                                _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
                            }
                0xF000 =>   match self.opcode & 0x00FF {
                                0x0007 => (), // vx = get_delay()
                                0x000A => (), // vx = get_key()
                                0x0015 => (), // delay_timer(vx)
                                0x0018 => (), // sound_timer(vx)
                                0x001E => (), // I += vx
                                0x0029 => (), // I = sprite_addr[vx]
                                0x0033 => (), // set_bcd(vx)
                                0x0055 => (), // reg_dump(vx, &I)
                                0x0065 => (), // reg_load(vx, &I)
                                _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
                            }
                _ => panic!("opcode decoded an unsupported code: {}!", self.opcode),
            }

            // Execute opcode

            // Update timers
        } 

        pub fn draw_flag(&self) -> bool {
            false
        }

        pub fn set_keys(&self) {

        }

        // returns from the subroutine
        fn return_subroutine(&mut self) {
            self.pc = self.stack[self.sp as usize]; 
            self.sp -= 1; 
        }

        // call the subroutine at the memory address NNN in opcode
        fn call_subroutine_at_nnn(&mut self) { 
            self.stack[self.sp as usize] = self.pc; 
            self.sp += 1; 
            self.pc = self.opcode & 0x0FFF; 
        }

        // skip the next instruction if Vx == NN
        fn skip_if_vx_equals_nn(&mut self) {
            if self.v[((self.opcode & 0x0FF0) >> 8) as usize] 
                    == (self.opcode & 0x00FF) as u8 {
                self.pc += 2;
            }
        }

        // skip the next instruction if Vx != NN
        fn skip_if_vx_not_equal_nn(&mut self) {
            if self.v[((self.opcode & 0x0FF0) >> 8) as usize] 
                    == (self.opcode & 0x00FF) as u8 {
                self.pc += 2;
            }
        }

        // skip the next instruction if Vx != Vy
        fn skip_if_vx_equals_vy(&mut self) {
            if self.v[((self.opcode & 0x0F00) >> 8) as usize] 
                    == self.v[((self.opcode & 0x00F0) >> 4) as usize] {
                self.pc += 2;
            }
        }

        // sets Vx equal to NN
        fn vx_equals_nn(&mut self) {
            self.v[((self.opcode & 0x0F00) >> 8) as usize] = (self.opcode & 0x00FF) as u8;
        }

        // add vx and nn and assign to vx
        fn vx_plus_equals_nn(&mut self) {
            self.v[((self.opcode & 0x0F00) >> 8) as usize] += (self.opcode & 0x00FF) as u8;
        }
    }
}
