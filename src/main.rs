extern crate sdl2;
mod cpu;

use std::env;
use std::process;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

use chip8::Config;
use chip8::{Palette, PALETTES};
use cpu::Chip8;

const WINDOW_WIDTH: u16 = 800;

pub fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    application(config);
}

fn application(config: Config) {
    let pixel_size: u8 = (WINDOW_WIDTH / 64) as u8;
    // Initialize SDL and Input Handling
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("chip-8-emu", pixel_size as u32 * 64, pixel_size as u32 * 32)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    // initially clear the screen
    let mut cur_color = 0;
    let mut draw_color = &PALETTES[cur_color];
    canvas.set_draw_color(draw_color.background);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize chip8 emulator
    let mut emu = Chip8::new();
    emu.load_game(&config.filename); // copy the program into memory

    'running: loop {
        // setup keys
        let mut keys: [u8; 16] = [0; 16];

        emu.emulate_cycle(); // Emulate one cycle

        if emu.draw_flag() {
            render(&mut emu, &mut canvas, &draw_color);
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(_w, _h),
                    ..
                } => render(&mut emu, &mut canvas, &draw_color),
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    cur_color = (cur_color + 1) % PALETTES.len();
                    draw_color = &PALETTES[cur_color];
                    render(&mut emu, &mut canvas, &draw_color);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => keys[1] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => keys[2] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => keys[3] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => keys[12] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => keys[4] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => keys[5] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => keys[6] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => keys[13] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => keys[7] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => keys[8] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => keys[9] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => keys[14] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => keys[10] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => keys[0] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => keys[11] = 255,
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => keys[15] = 255,
                _ => {}
            }
        }
        emu.set_keys(&keys);

        ::std::thread::sleep(Duration::new(0, 100_000_000u32 / 60));
    }
}

// Handles drawing the Chip8 video ram to the SDL2 window.
fn render(emu: &mut Chip8, canvas: &mut Canvas<Window>, draw_color: &Palette) {
    // TODO: abstract away directly accessing array
    //if emu.draw_flag() {
    canvas.set_draw_color(draw_color.gutter);
    canvas.clear();
    let pixel_size = canvas.window().size().0 / 64;
    let gutter: i32 =
        (canvas.window().size().1 as i32 - (pixel_size as i32 * 32)) as i32 / 2 as i32;

    canvas.set_draw_color(draw_color.background);
    let _result = canvas.fill_rect(Rect::new(
        0,
        gutter,
        canvas.window().size().0,
        (canvas.window().size().1 as i32 - (2 * gutter as i32)) as u32,
    ));
    // loop through the pixel array
    for x in 0..63 {
        for y in 0..31 {
            // Only draw the pixel if its on
            if emu.gfx[y][x] != 0 {
                // get the x and y coordinate in screen space
                let x: i32 = x as i32 * pixel_size as i32;
                let y: i32 = (y as i32 * pixel_size as i32) + gutter as i32;

                canvas.set_draw_color(draw_color.foreground);
                let _result =
                    canvas.fill_rect(Rect::new(x, y, pixel_size.into(), pixel_size.into()));
            }
        }
    }
    canvas.present();
    //}
}
