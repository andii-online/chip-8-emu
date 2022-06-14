extern crate sdl2;
mod cpu;

use std::env;
use std::process;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

use chip8::Config;
use cpu::Chip8;

const WINDOW_WIDTH: u16 = 400;
const PIXEL_SIZE: u8 = (WINDOW_WIDTH / 64) as u8;

pub fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    application(config);
}

pub fn application(config: Config) {
    // Initialize SDL and Input Handling
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("chip-8-emu", PIXEL_SIZE as u32 * 64, PIXEL_SIZE as u32 * 32)
        .resizable()
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
    let mut emu = Chip8::new();
    emu.load_game(&config.filename); // copy the program into memory

    'running: loop {
        emu.emulate_cycle(); // Emulate one cycle

        if emu.draw_flag() {
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            // TODO: abstract away directly accessing array
            // loop through the pixel array
            for x in 0..63 {
                for y in 0..31 {
                    // Only draw the pixel if its on
                    if emu.gfx[y][x] != 0 {
                        // get the x and y coordinate in screen space
                        let x: i32 = x as i32 * PIXEL_SIZE as i32;
                        let y: i32 = y as i32 * PIXEL_SIZE as i32;

                        canvas.set_draw_color(Color::WHITE);
                        let _result =
                            canvas.fill_rect(Rect::new(x, y, PIXEL_SIZE.into(), PIXEL_SIZE.into()));
                    }
                }
            }
            canvas.present();
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        emu.set_keys();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
