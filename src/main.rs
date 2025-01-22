use std::env;
use std::process;

use sasm2::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut config = Config::build(&args).unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });

    if let Err(s) = sasm2::run(&mut config) {
        eprintln!("{s}");
    }
}
