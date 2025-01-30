use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut config = sasm2::Config::build(&args).unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });

    if let Err(s) = sasm2::assemble(&mut config) {
        eprintln!("{s}");
    }
}
