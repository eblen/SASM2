mod data;
mod output;
mod syntax;
mod zpm;

use indoc::indoc;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

use crate::data::*;
use crate::output::*;
use crate::syntax::*;
use crate::zpm::*;

pub use crate::output::Code;

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

pub fn help() -> &'static str {
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

fn write_code_to_file<T: std::convert::AsRef<[u8]>>(f: &str, c: T) -> Result<(), String> {
    match std::fs::exists(f) {
        Ok(true) => Err(format!("File {f} already exists")),
        Ok(false) => match std::fs::write(f, c) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Unable to write to file {f}")),
        },
        Err(_) => Err(format!("Unable to check existence of file {f}")),
    }
}

pub fn run(config: &mut Config) -> Result<Code, String> {
    let assembly = match config.itype {
        IType::Stdin => {
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .expect("Unable to read from stdin");
            s
        }
        IType::String(ref s) => s.to_string(),
        IType::File(ref f) => std::fs::read_to_string(f).expect("Unable to read input file"),
    };

    // Main data structures
    // Vector of tokenized source lines
    let mut source = Vec::new();

    // Map of label names to value
    let mut labels = HashMap::new();

    // Current code address (address where the current byte will be stored in memory)
    let mut code_addr: u16 = 0;

    // Current code position (position of current byte in assembly code, which is unchanged by
    // "org" statements)
    let mut code_pos: u16 = 0;

    // Map of org values to code positions
    let mut org_to_code_pos = BTreeMap::new();

    // Insert a default, initial org of 0000. Thus, an org statement is not required before code,
    // although most programs should have one. (One exception is code for testing SASM itself.)
    // If an org statement does appear before any code, this entry will be removed.
    org_to_code_pos.insert(0, 0);

    // First parser loop. Tokenizes source lines and collects labels.
    for (line_num, line) in assembly.lines().enumerate() {
        match tokenize(line) {
            Ok(tokenized_line) => {
                match tokenized_line {
                    SourceLine::Blank => (),
                    SourceLine::Org(o) => {
                        if o < code_addr {
                            return Err(format!(
                                "{line_num}: Org smaller than code address: {:x}",
                                code_addr
                            ));
                        }

                        // If org appears before any code, remove the default, initial org.
                        if code_pos == 0 {
                            org_to_code_pos.clear();
                        }

                        org_to_code_pos.insert(o, code_pos);
                        code_addr = o;
                    }
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
                        code_addr += d.len() as u16;
                        code_pos += d.len() as u16;
                    }
                    SourceLine::CodeMarker(ref s) => {
                        if labels.contains_key(s) {
                            return Err(format!("{line_num}: Label repeated"));
                        }
                        labels.insert(s.to_string(), UInt::U16(code_addr));
                    }
                    SourceLine::Instr(ref mnemonic, _, _) => {
                        code_addr += get_instr_size(mnemonic)? as u16;
                        code_pos += get_instr_size(mnemonic)? as u16;
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
    code_addr = 0;
    let mut disassembly: Vec<u8> = Vec::new();
    for s in source {
        match s {
            SourceLine::Org(o) => {
                code_addr = o;
            }
            SourceLine::Data(d) => disassembly.extend(d),
            SourceLine::Instr(mnemonic, input_op, offset_type) => {
                // Store opcode
                let instr_info = get_instr_info(&mnemonic)?;
                disassembly.push(instr_info.opcode);
                code_addr += 1;

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
                                    code_addr += 1;
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
                                        // Jump is from the end of the current instruction
                                        // (code_addr + 1)
                                        match compute_diff_u16_as_u8(
                                            u + offset as u16,
                                            code_addr + 1,
                                        ) {
                                            Some(d) => {
                                                disassembly.push(d);
                                                code_addr += 1;
                                            }
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
                                    code_addr += 2;
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

    // Create output and print to STDOUT or file if requested.
    let code = output::bytes_to_output(&disassembly, org_to_code_pos, config.cformat);
    match code {
        Code::String(ref s) => match &config.otype {
            OType::Stdout => println!("{s}"),
            OType::File(f) => {
                if let Err(e) = write_code_to_file(f, &s) {
                    return Err(format!("Error: {e}"));
                }
            }
            OType::None => (),
        },
        Code::Bytes(ref b) => match &config.otype {
            OType::Stdout => std::io::stdout()
                .write_all(&b)
                .expect("Unable to write binary to stdout"),
            OType::File(f) => {
                if let Err(e) = write_code_to_file(f, &b) {
                    return Err(format!("Error: {e}"));
                }
            }
            OType::None => (),
        },
    }

    return Ok(code);
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
