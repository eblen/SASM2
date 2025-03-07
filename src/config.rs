use indoc::indoc;

use crate::output::CodeFormat;
use crate::zpm::Zpm;

pub enum IType {
    Stdin,
    String(String),
    File(String),
}

pub enum OType {
    Stdout,
    File(String),
    None,
}

pub struct Config {
    pub itype: IType,
    pub otype: OType,
    pub zpm: Zpm,
    pub cformat: CodeFormat,
    pub addr: u16,
    pub min_region_size: usize,
}

fn help() -> &'static str {
    return indoc! {"
            Flags (all are optional):
            -h: This help message
            -i: Input  file (STDIN  is default)
            -o: Output file (STDOUT is default)
            -s: System: (assembler only)
                apple: Apple II (default)
                atari: Atari 2600
            -f: Code output format: (assembler only)
                hex:   String of hex digits (default)
                apple: Apple II system monitor
                bin:   Machine code
            -a: Starting address in hex (disassembler only)
                0x0000 is default. Must be < 0x10000.
            -m: Minimum size for a code region (disassembler only)
                10 is default.
    "};
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, String> {
        // Flags to keep track of state while parsing the command line.
        enum CLFlag {
            Ifile,
            Ofile,
            Sys,
            Format,
            Addr,
            MinRegSize,
            None,
        }

        // Config with default values. Only zpm must be changed before build completes.
        let mut config = Config {
            itype: IType::Stdin,
            otype: OType::Stdout,
            zpm: Zpm::None, // Defaults to AppleII
            cformat: CodeFormat::Hex,
            addr: 0,
            min_region_size: 10,
        };

        // Simple but strict argument parser. All flags are optional.
        let mut current_flag = CLFlag::None;
        let mut args_iter = args.iter();
        _ = args_iter.next();
        for a in args_iter {
            // Process flags
            if a.starts_with('-') {
                if let CLFlag::None = current_flag {
                    match a.as_str() {
                        "-h" => return Err(help().to_string()),
                        "-i" => current_flag = CLFlag::Ifile,
                        "-o" => current_flag = CLFlag::Ofile,
                        "-s" => current_flag = CLFlag::Sys,
                        "-f" => current_flag = CLFlag::Format,
                        "-a" => current_flag = CLFlag::Addr,
                        "-m" => current_flag = CLFlag::MinRegSize,
                        _ => return Err(format!("Invalid flag: {a}")),
                    }
                } else {
                    return Err(format!("Flag {a} cannot follow another flag"));
                }

            // Process arguments
            } else {
                match current_flag {
                    CLFlag::Ifile => config.itype = IType::File(a.to_string()),
                    CLFlag::Ofile => config.otype = OType::File(a.to_string()),
                    CLFlag::Sys => config.zpm = Zpm::new(a)?,
                    CLFlag::Format => config.cformat = CodeFormat::new(a)?,
                    CLFlag::Addr => {
                        config.addr = match u16::from_str_radix(&a, 16) {
                            Ok(n) => n,
                            _ => return Err("Invalid starting address".to_string()),
                        }
                    }
                    CLFlag::MinRegSize => {
                        config.min_region_size = match a.parse() {
                            Ok(n) => n,
                            _ => return Err("Invalid minimum region size".to_string()),
                        }
                    }
                    CLFlag::None => {
                        return Err(format!("Argument {a} must immediately follow a flag"))
                    }
                }

                current_flag = CLFlag::None;
            }
        }

        // Default system is Apple II (currently only sets the zero-page manager).
        if let Zpm::None = config.zpm {
            config.zpm = Zpm::new_for_apple();
        }

        // Check for illegal combinations
        match config.zpm {
            Zpm::Atari2600 { .. } => match config.cformat {
                CodeFormat::AppleSM => {
                    return Err("Apple System Monitor output not compatible with Atari".to_string())
                }
                _ => (),
            },
            _ => (),
        }

        return Ok(config);
    }

    pub fn build_string_test(input_string: &str) -> Config {
        Config {
            itype: IType::String(input_string.to_string()),
            otype: OType::None,
            zpm: Zpm::new_for_apple(),
            cformat: CodeFormat::Hex,
            addr: 0,
            min_region_size: 10,
        }
    }
}
