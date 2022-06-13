extern crate sdl2;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

mod chip8;
use chip8::chip8 as c8;

const PIXEL_SIZE: u8 = 20;

pub fn main() {
    // Initialize SDL and Input Handling
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window(
        "chip-8-emu", 
        PIXEL_SIZE as u32 * 64, 
        PIXEL_SIZE as u32 * 32
    )
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    // initially clear the screen
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize chip8 emulator
    let mut emu = c8::init(); 
    emu.load_game("ibm-logo.ch8"); // copy the program into memory

    'running: loop {
        emu.emulate_cycle(); // Emulate one cycle
        
        if emu.draw_flag() {
            // clear the screen!
            canvas.set_draw_color(Color::BLACK); 
            canvas.clear();

            // TODO: abstract away directly accessing array
            // loop through the pixel array
            for (i, pix) in emu.gfx.iter().enumerate() {
                // Only draw the pixel if its on
                if *pix != 0 {
                    // get the x and y coordinate in screen space
                    let x: i32 = (i % 64) as i32 * PIXEL_SIZE as i32;
                    let y: i32 = ((i / 64) % 32) as i32 * PIXEL_SIZE as i32;

                    canvas.set_draw_color(Color::WHITE);
                    let _result = canvas.fill_rect(Rect::new(
                            x, 
                            y, 
                            PIXEL_SIZE.into(), 
                            PIXEL_SIZE.into()
                    ));
                }
            }
            canvas.present();
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
        emu.set_keys();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

