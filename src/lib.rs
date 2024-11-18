use std::error::Error;
use std::fs;

mod types;
use crate::types::*;

pub struct Config {
    pub input_file_path: String,
    pub output_file_path: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &str> {
        // Should have been checked by main
        assert!(args.len() > 1);

        if args.len() < 3 {
            return Err("Missing output file path");
        }

        let input_file_path = args[1].clone();
        let output_file_path = args[2].clone();
        Ok(Config {
            input_file_path,
            output_file_path,
        })
    }
}

pub fn help() -> &'static str {
    return "Usage: sasm <assembly input file> <binary output file>";
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
        _ => Err("not a valid keyword"),
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.input_file_path)?;

    for (line_num, line) in contents.lines().enumerate() {
        match tokenize(line) {
            Ok(_) => todo!(),
            Err(s) => return Err(format!("{line_num}: {s}").into()),
        }
    }

    Ok(())
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
