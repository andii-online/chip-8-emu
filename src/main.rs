extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

mod chip8;
use chip8::chip8 as c8;

pub fn main() {
    // Initialize SDL and Input Handling
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("chip-8-emu", 800, 400)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize chip8 emulator
    let mut emu = c8::init(); 
    emu.load_game("ibm-logo.ch8"); // copy the program into memory

    'running: loop {
        println!("emulation cycle");
        emu.emulate_cycle(); // Emulate one cycle

        if emu.draw_flag() {
            println!("rendering the frame");
            render(&emu);
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                },
                _ => {}
            }
        }
        println!("setting pressed keys");
        emu.set_keys();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn render(emu: &c8::Chip8) {
    todo!();
}
