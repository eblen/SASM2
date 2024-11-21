use std::env;
use std::process;

use sasm2::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("{}", sasm2::help());
        process::exit(1);
    }

    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });

    match sasm2::run(config) {
        Ok(_) => println!("Assembly complete"),
        Err(s) => println!("{s}"),
    }
}
