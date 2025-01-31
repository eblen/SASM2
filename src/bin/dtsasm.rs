use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Configuration is the same for assembly and disassmbly, but disassembly
    // ignores the -s (system) and -f (format) flags.
    let mut config = sasm2::Config::build(&args).unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });

    if let Err(s) = sasm2::disassemble(&mut config) {
        eprintln!("{s}");
    }
}
