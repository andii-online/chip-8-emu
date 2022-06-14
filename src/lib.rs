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
