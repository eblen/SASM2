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
}

fn help() -> &'static str {
    return indoc! {"
            Flags (all are optional):
            -h: This help message
            -i: Input  file (STDIN  is default)
            -o: Output file (STDOUT is default)
            -s: System:
                apple: Apple II (default)
                atari: Atari 2600
            -f: Code output format:
                hex:   String of hex digits (default)
                apple: Apple II system monitor
                bin:   Machine code
    "};
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &str> {
        // Flags to keep track of state while parsing the command line.
        enum CLFlag {
            Ifile,
            Ofile,
            Sys,
            Format,
            None,
        }

        // Config with default values. Only zpm must be changed before build completes.
        let mut config = Config {
            itype: IType::Stdin,
            otype: OType::Stdout,
            zpm: Zpm::None, // Defaults to AppleII
            cformat: CodeFormat::Hex,
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
                        "-h" => return Err(help()),
                        "-i" => current_flag = CLFlag::Ifile,
                        "-o" => current_flag = CLFlag::Ofile,
                        "-s" => current_flag = CLFlag::Sys,
                        "-f" => current_flag = CLFlag::Format,
                        _ => return Err("Invalid flag: {a}"),
                    }
                } else {
                    return Err("Flag {a} cannot follow another flag");
                }

            // Process arguments
            } else {
                match current_flag {
                    CLFlag::Ifile => config.itype = IType::File(a.to_string()),
                    CLFlag::Ofile => config.otype = OType::File(a.to_string()),
                    CLFlag::Sys => config.zpm = Zpm::new(a)?,
                    CLFlag::Format => config.cformat = CodeFormat::new(a)?,
                    CLFlag::None => return Err("Argument {a} must immediately follow a flag"),
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
                    return Err("Apple System Monitor output not compatible with Atari")
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
        }
    }
}
