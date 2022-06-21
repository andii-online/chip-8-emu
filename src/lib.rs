use sdl2::pixels::Color;
use std::env;

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        // the first arg is always the name of the command that executed
        // this program
        args.next();

        if args.len() > 2 {
            return Err("Not enough arguments");
        }

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a filename string"),
        };

        Ok(Config { filename })
    }
}

pub struct Palette {
    pub background: Color,
    pub foreground: Color,
    pub gutter: Color,
}

const DEFAULT_PALETTE: Palette = Palette {
    background: Color::RGB(34, 35, 35),
    foreground: Color::RGB(240, 246, 240),
    gutter: Color::RGB(255 - 34, 255 - 35, 255 - 35),
};

const BITBEE: Palette = Palette {
    background: Color::RGB(41, 43, 48),
    foreground: Color::RGB(207, 171, 74),
    gutter: Color::RGB(255 - 41, 255 - 43, 255 - 48),
};

const NEUTRAL_GREEN: Palette = Palette {
    background: Color::RGB(0, 76, 61),
    foreground: Color::RGB(255, 234, 249),
    gutter: Color::RGB(255, 255 - 76, 255 - 61),
};

const MAC_PAINT: Palette = Palette {
    background: Color::RGB(139, 200, 254),
    foreground: Color::RGB(5, 27, 44),
    gutter: Color::RGB(255 - 139, 255 - 200, 255 - 254),
};

const PAPER_BACK: Palette = Palette {
    background: Color::RGB(184, 194, 185),
    foreground: Color::RGB(56, 43, 38),
    gutter: Color::RGB(255 - 184, 255 - 194, 255 - 185),
};

pub const PALETTES: [Palette; 5] = [
    DEFAULT_PALETTE,
    BITBEE,
    NEUTRAL_GREEN,
    MAC_PAINT,
    PAPER_BACK,
];
