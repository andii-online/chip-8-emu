extern crate sdl2;
mod cpu;

use std::env;
use std::process;

use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use chip8::Config;
use chip8::{Palette, PALETTES, DEFAULT_PALETTE};
use cpu::Chip8;

const WINDOW_WIDTH: u16 = 800;
const EMULATOR_WIDTH: u8 = 64;
const EMULATOR_HEIGHT: u8 = 32;

pub fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("❌ Problem parsing arguments: {}", err);
        process::exit(1);
    });

    application(config);
}

fn application(config: Config) {
    let pixel_size: u8 = (WINDOW_WIDTH / EMULATOR_WIDTH as u16) as u8;
    // Initialize SDL and Input Handling
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("chip-8-emu", 
            pixel_size as u32 * EMULATOR_WIDTH as u32, 
            pixel_size as u32 * EMULATOR_HEIGHT as u32)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    // initially clear the screen
    let mut color_palette: &Palette = &DEFAULT_PALETTE; 
    canvas.set_draw_color(color_palette.background);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Initialize chip8 emulator
    let mut emu = Chip8::default();
    // copy the program into memory
    match emu.load_game(&config.rom_path) {
        Err(e) => {
            eprint!("❌ Error loading ROM file {e:?}.");
            std::process::exit(1);
        }
        _ => (),
    };

    'running: loop {
        // setup keys
        let mut keys: [u8; 16] = [0; 16];

        emu.emulate_cycle(); // Emulate one cycle

        if emu.draw_flag() {
            render(&emu, &mut canvas, &color_palette);
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
                } => render(&mut emu, &mut canvas, &color_palette),
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    next_palette(&mut color_palette);
                    render(&mut emu, &mut canvas, &color_palette);
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

        //::std::thread::sleep(Duration::new(0, 100_000_000u32 / 6000));
    }
}

// Draws the current gfx buffer onto the Canvas. 
// 
// I'm not crazy about this abstraction...
fn render(emu: &Chip8, canvas: &mut Canvas<Window>, draw_color: &Palette) {
    let screen_width = canvas.window().size().0;
    let screen_height = canvas.window().size().1; 

    // Clear screen for gutters
    canvas.set_draw_color(draw_color.gutter);
    canvas.clear();

    // Recalculate constants for the current window size
    let pixel_size = screen_width / 64;
    let gutter: i32 =
        (screen_height as i32 - (pixel_size as i32 * EMULATOR_HEIGHT as i32)) as i32 / 2 as i32;

    canvas.set_draw_color(draw_color.background);
    let _result = canvas.fill_rect(Rect::new(
        0,
        gutter,
        screen_width,
        (screen_height as i32 - (2 * gutter as i32)) as u32,
    ));

    // loop through the pixel array
    for x in 0..EMULATOR_WIDTH {
        for y in 0..EMULATOR_HEIGHT {
            // Only draw the pixel if its on
            if emu.gfx[y as usize][x as usize] != 0 {
                // get the x and y coordinate in screen space
                let screen_x: i32 = x as i32 * pixel_size as i32;
                let screen_y: i32 = (y as i32 * pixel_size as i32) + gutter as i32;

                canvas.set_draw_color(draw_color.foreground);
                let _result =
                    canvas.fill_rect(Rect::new(screen_x, screen_y, pixel_size.into(), pixel_size.into()));
            }
        }
    }
    canvas.present();
}

//TODO: write this helper function.
fn next_palette(mut _curr_palette: &Palette) {
    _curr_palette = &PALETTES[0];
}
