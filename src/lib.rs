mod data;
mod output;
mod syntax;
mod zpm;

use std::collections::HashMap;

use crate::data::*;
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

// Compute x-y and return only if result fits in an i8.
// However, return it as a u8 (same bits) so that it can be stored in a disassembly.
// This is a bit tricky in Rust, so we write a separate function.
fn compute_diff_u16_as_u8(x: u16, y: u16) -> Option<u8> {
    let diff: i32 = x as i32 - y as i32;
    if diff > 127 || diff < -128 {
        return None;
    }

    // A single byte, so endianness doesn't matter
    return Some(diff.to_ne_bytes()[0]);
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
            2 => Ok(SourceLine::ZByte(words[1].to_string(), 1)),
            3 => match hex_to_uint(words[2])? {
                UInt::U8(u) => Ok(SourceLine::ZByte(words[1].to_string(), u)),
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

        // Code markers
        cm if cm.starts_with('.') => {
            if words.len() != 1 {
                return Err("Code markers must be on a line by themselves");
            }
            Ok(SourceLine::CodeMarker(words[0][1..].to_string()))
        }

        // Assume an instruction
        _ => {
            // Tokenize operand
            let mut op = Op::None;
            if words.len() > 1 {
                op = if words[1].starts_with('.') {
                    Op::Label(words[1][1..].to_string())
                } else {
                    Op::UInt(hex_to_uint(words[1])?)
                }
            }

            // Tokenize offset
            let mut offset = Offset::U8(0);
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

pub fn run(config: &mut Config) -> Result<String, String> {
    let assembly = match config.itype {
        IType::STRING(s) => s,
        IType::FILE(ref s) => &std::fs::read_to_string(s).expect("Unable to read input file"),
    };

    // Main data structures
    // Vector of tokenized source lines
    let mut source = Vec::new();

    // Map of label names to value
    let mut labels = HashMap::new();

    // Current code byte
    let mut code_byte: u16 = 0;

    // First parser loop. Tokenizes source lines and collects labels.
    for (line_num, line) in assembly.lines().enumerate() {
        match tokenize(line) {
            Ok(tokenized_line) => {
                match tokenized_line {
                    SourceLine::Blank => (),
                    SourceLine::Org(_) => (),
                    SourceLine::Label(ref s, u) => {
                        if labels.contains_key(s) {
                            return Err(format!("{line_num}: Label repeated"));
                        }
                        labels.insert(s.to_string(), u);
                    }
                    SourceLine::ZByte(ref s, size) => {
                        if labels.contains_key(s) {
                            return Err(format!("{line_num}: Label repeated"));
                        }
                        labels.insert(s.to_string(), UInt::U8(config.zpm.alloc(size)));
                    }
                    SourceLine::Data(ref d) => {
                        code_byte += d.len() as u16;
                    }
                    SourceLine::CodeMarker(ref s) => {
                        if labels.contains_key(s) {
                            return Err(format!("{line_num}: Label repeated"));
                        }
                        labels.insert(s.to_string(), UInt::U16(code_byte));
                    }
                    SourceLine::Instr(ref mnemonic, _, _) => {
                        code_byte += get_instr_size(mnemonic)? as u16;
                    }
                }

                // Store all source lines so that next loop can refer to input
                // by line number.
                source.push(tokenized_line);
            }
            Err(s) => return Err(format!("{line_num}: {s}")),
        };
    }

    // Second parser loop. Stores machine code in "disassembly" vector.
    let mut disassembly: Vec<u8> = Vec::new();
    for s in source {
        match s {
            SourceLine::Org(_) => (),
            SourceLine::Data(d) => disassembly.extend(d),
            SourceLine::Instr(mnemonic, input_op, offset_type) => {
                // Store opcode
                let instr_info = get_instr_info(&mnemonic)?;
                disassembly.push(instr_info.opcode);

                // Compute offset
                let offset: u8;
                match offset_type {
                    Offset::U8(u) => offset = u,
                    Offset::Label(l) => match labels.get(&l) {
                        Some(UInt::U8(u)) => offset = *u,
                        Some(UInt::U16(_)) => {
                            return Err("Offset must be a single byte".to_string())
                        }
                        None => panic!("Internal error: label found in second pass but not first"),
                    },
                }

                // Handle labelled op. Unwrap it and convert it to a non-label variant.
                let input_op_unwrapped: Op;
                if let Op::Label(l) = input_op {
                    input_op_unwrapped = match labels.get(&l) {
                        Some(u) => Op::UInt(*u),
                        None => panic!("Internal error: label found in second pass but not first"),
                    }
                } else {
                    input_op_unwrapped = input_op;
                }

                // Handle op
                match input_op_unwrapped {
                    Op::Label(_) => panic!("Internal error: label found for unwrapped op"),

                    // No operand provided
                    Op::None => match instr_info.op {
                        OpType::None => (),
                        OpType::U8 => {
                            return Err("Instruction requires a single-byte operand".to_string())
                        }
                        OpType::U16 => {
                            return Err("Instruction requires a two-byte operand".to_string())
                        }
                    },

                    // UInt op provided (recall that labels have already been unwrapped)
                    Op::UInt(ui_type) => match ui_type {
                        // UInt op is a single byte
                        UInt::U8(u) => match instr_info.op {
                            OpType::None => {
                                return Err("Instruction does not require an operand".to_string())
                            }
                            OpType::U8 => {
                                if u as u16 + offset as u16 > 0xff {
                                    return Err("Operand plus offset is > 0xff".to_string());
                                } else {
                                    disassembly.push(u + offset);
                                }
                            }
                            OpType::U16 => {
                                return Err("Instruction requires a two-byte operand".to_string())
                            }
                        },

                        // UInt op is two bytes
                        UInt::U16(u) => match instr_info.op {
                            OpType::None => {
                                return Err("Instruction does not require an operand".to_string())
                            }
                            OpType::U8 => {
                                // Special handling for relative branches. Allow them to have a
                                // two-byte operand from which we compute the real, single-byte
                                // operand (a code offset). Normally this will come from a label.

                                // Note that it is possible for the user to hardcode the relative
                                // offset by giving a single-byte operand.
                                if is_relative_branch_instruction(&mnemonic) {
                                    // Not sure if it makes sense to support offsets here, but they are
                                    // not forbidden anywhere else, so let's be consistent.
                                    if u as u32 + offset as u32 > 0xffff {
                                        return Err("Operand plus offset is > 0xffff".to_string());
                                    } else {
                                        // Should be between opcode and op
                                        let current_code_pos = disassembly.len() as u16;

                                        // Jump is from the end of the current instruction
                                        match compute_diff_u16_as_u8(
                                            u + offset as u16,
                                            current_code_pos + 1,
                                        ) {
                                            Some(d) => disassembly.push(d),
                                            None => {
                                                return Err(
                                                    "Relative branch is too far from target"
                                                        .to_string(),
                                                )
                                            }
                                        }
                                    }
                                } else {
                                    return Err(
                                        "Instruction requires a single-byte operand".to_string()
                                    );
                                }
                            }
                            OpType::U16 => {
                                if u as u32 + offset as u32 > 0xffff {
                                    return Err("Operand plus offset is > 0xffff".to_string());
                                } else {
                                    let bytes = (u + offset as u16).to_le_bytes();
                                    disassembly.push(bytes[0]);
                                    disassembly.push(bytes[1]);
                                }
                            }
                        },
                    },
                }
            }

            // All other line types ignored in second pass
            _ => (),
        }
    }

    let s = output::hex_format(&disassembly);

    match &config.otype {
        OType::STDOUT => println!("{s}"),
        OType::STRING => return Ok(s),
        OType::FILE(f) => {
            if let Err(e) = write_assembly_to_file(f, &s) {
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
