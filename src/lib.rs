mod output;
mod syntax;
mod zpm;
use crate::syntax::*;
use crate::zpm::*;

pub enum IType<'a> {
    STRING(&'a str),
    FILE(String),
}

pub enum OType {
    STDOUT,
    STRING,
    FILE(String),
}

pub struct Config<'a> {
    pub itype: IType<'a>,
    pub otype: OType,
    pub zpm: Zpm,
}

impl Config<'_> {
    pub fn build(args: &[String]) -> Result<Config, &str> {
        // Should have been checked by main
        assert!(args.len() > 1);

        if args.len() < 3 {
            return Err(help());
        }

        let sys: Zpm;
        if args.len() > 3 {
            sys = Zpm::new(&args[3])?;
        } else {
            sys = Zpm::new_for_apple();
        }

        println!("{:?}", sys);

        Ok(Config {
            itype: IType::FILE(args[1].clone()),
            otype: OType::FILE(args[2].clone()),
            zpm: sys,
        })
    }

    pub fn build_string_test<'a>(input_string: &'a str) -> Config<'a> {
        Config {
            itype: IType::STRING(input_string),
            otype: OType::STRING,
            zpm: Zpm::new_for_apple(),
        }
    }
}

pub fn help() -> &'static str {
    return "Usage: sasm <assembly input file> <binary output file> <system (optional)>\n\
            Possible systems: appleII (default) or atari2600";
}

fn hex_to_uint(s: &str) -> Result<UInt, &str> {
    let num_hex_digits = s.len();
    let em = "not a valid hexadecimal number";

    match num_hex_digits {
        1 | 2 => match u8::from_str_radix(&s, 16) {
            Ok(n) => Ok(UInt::U8(n)),
            _ => Err(em),
        },

        4 => match u16::from_str_radix(&s, 16) {
            Ok(n) => Ok(UInt::U16(n)),
            _ => Err(em),
        },

        _ => Err(em),
    }
}

pub fn tokenize(line: &str) -> Result<SourceLine, &str> {
    // Remove comments
    let words: Vec<&str> = line
        .splitn(1, "//")
        .next()
        .unwrap()
        .split_ascii_whitespace()
        .collect();
    if words.len() == 0 {
        return Ok(SourceLine::Blank);
    }

    match words[0] {
        "org" => {
            if words.len() != 2 {
                return Err("org takes one argument");
            }
            match hex_to_uint(words[1])? {
                UInt::U8(_) => Err("org must be a 2-byte address"),
                UInt::U16(u) => Ok(SourceLine::Org(u)),
            }
        }

        "label" => {
            if words.len() != 3 {
                return Err("label takes two arguments");
            }

            match hex_to_uint(words[2])? {
                UInt::U8(u) => Ok(SourceLine::Label(words[1].to_string(), UInt::U8(u))),
                UInt::U16(u) => Ok(SourceLine::Label(words[1].to_string(), UInt::U16(u))),
            }
        }

        "zbyte" => match words.len() {
            2 => Ok(SourceLine::Zbyte(words[1].to_string(), 1)),
            3 => match hex_to_uint(words[2])? {
                UInt::U8(u) => Ok(SourceLine::Zbyte(words[1].to_string(), u)),
                UInt::U16(_) => Err("zbyte array size must be a single byte (< 0x100)"),
            },
            _ => Err("zbyte takes one or two arguments"),
        },

        "data" => {
            if words.len() != 2 {
                return Err("data takes one argument");
            }
            match hex::decode(words[1]) {
                Ok(v) => Ok(SourceLine::Data(v)),
                Err(_) => Err("data must be a valid hex string"),
            }
        }

        // Assume an instruction
        _ => {
            let mut op = Op::None;
            let mut offset = Offset::None;

            // Tokenize operand
            if words.len() > 1 {
                op = if words[1].starts_with('.') {
                    Op::Label(words[1][1..].to_string())
                } else {
                    Op::UInt(hex_to_uint(words[1])?)
                }
            }

            // Tokenize offset
            if words.len() > 2 {
                offset = if words[2].starts_with('.') {
                    Offset::Label(words[2][1..].to_string())
                } else {
                    Offset::U8(match hex_to_uint(words[2])? {
                        UInt::U8(u) => u,
                        UInt::U16(_) => return Err("offset must be a single byte (< 0x100)"),
                    })
                }
            }

            Ok(SourceLine::Instr(words[0].to_string(), op, offset))
        }
    }
}

fn write_assembly_to_file(f: &str, s: &str) -> Result<(), String> {
    match std::fs::exists(f) {
        Ok(true) => Err(format!("File {f} already exists")),
        Ok(false) => match std::fs::write(f, s) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Unable to write to file {f}")),
        },
        Err(_) => Err(format!("Unable to check existence of file {f}")),
    }
}

pub fn run(config: Config) -> Result<String, String> {
    let assembly = match config.itype {
        IType::STRING(s) => s,
        IType::FILE(s) => &std::fs::read_to_string(s).expect("Unable to read input file"),
    };

    let mut source = Vec::new();
    for (line_num, line) in assembly.lines().enumerate() {
        match tokenize(line) {
            Ok(s) => source.push(s),
            Err(s) => return Err(format!("{line_num}: {s}")),
        };
    }

    let s = output::hex_format(&source);

    match config.otype {
        OType::STDOUT => println!("{s}"),
        OType::STRING => return Ok(s),
        OType::FILE(f) => {
            if let Err(e) = write_assembly_to_file(&f, &s) {
                return Err(format!("Error: {e}"));
            }
        }
    }

    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex2_to_u8() {
        match hex_to_uint("EB") {
            Ok(UInt::U8(i)) => assert_eq!(i, 235),
            _ => panic!("Unable to convert single-byte hex to int"),
        }
    }

    #[test]
    fn hex4_to_u16() {
        match hex_to_uint("AD80") {
            Ok(UInt::U16(i)) => assert_eq!(i, 44416),
            _ => panic!("Unable to convert 2-byte hex to int"),
        }
    }

    #[test]
    fn hex3_to_err() {
        let e = hex_to_uint("D80");
        assert!(e.is_err());
    }

    #[test]
    fn non_hex_to_err() {
        let e = hex_to_uint("John");
        assert!(e.is_err());
    }
}
