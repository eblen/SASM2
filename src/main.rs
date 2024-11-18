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

    if let Err(e) = sasm2::run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}
