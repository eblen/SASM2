use std::env;
use std::process;

use sasm2::{Config, OType};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("{}", sasm2::help());
        process::exit(1);
    }

    let mut config = Config::build(&args).unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });
    let should_print = matches!(config.otype, OType::STRING);

    match sasm2::run(&mut config) {
        Ok(s) => {
            if should_print {
                println!("{s}")
            }
        }
        Err(s) => eprintln!("{s}"),
    }
}
