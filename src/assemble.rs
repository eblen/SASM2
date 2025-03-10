use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Read;

use crate::config::*;
use crate::data::*;
use crate::output::*;
use crate::syntax::*;

fn hex_to_uint(s: &str) -> Result<UInt, &str> {
    let num_hex_digits = s.len();
    let em = "not a valid hexadecimal number";

    match num_hex_digits {
        1 | 2 => match u8::from_str_radix(&s, 16) {
            Ok(n) => Ok(UInt::U8(n)),
            _ => Err(em),
        },

        3 | 4 => match u16::from_str_radix(&s, 16) {
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

fn tokenize(line: &str) -> Result<SourceLine, &str> {
    // Remove comments
    let words: Vec<&str> = line
        .split(";")
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
            if words[1].starts_with('.') {
                Ok(SourceLine::Data(Rawdata::Label(words[1][1..].to_string())))
            } else {
                match hex::decode(words[1]) {
                    Ok(v) => Ok(SourceLine::Data(Rawdata::Bytes(v))),
                    Err(_) => Err("data must be a valid hex string"),
                }
            }
        }

        // Code markers
        cm if cm.starts_with('.') => {
            if words.len() != 1 {
                return Err("code markers must be on a line by themselves");
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

// This parent function allows us to easily append the line number to any errors regardless of how
// and where they are generated.
pub fn assemble(config: &mut Config) -> Result<Code, String> {
    let mut line_num = 0;
    match run_internal(config, &mut line_num) {
        Ok(c) => Ok(c),
        Err(e) => Err(format!("{line_num}: {e}")),
    }
}

fn run_internal(config: &mut Config, line_num: &mut i32) -> Result<Code, String> {
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
    let mut code_addr: usize = 0;

    // Current code position (position of current byte in assembly code, which is unchanged by
    // "org" statements)
    let mut code_pos: usize = 0;

    // Map of org values to code positions
    let mut org_to_code_pos = BTreeMap::new();

    // Insert a default, initial org of 0000. Thus, an org statement is not required before code,
    // although most programs should have one. (One exception is code for testing SASM itself.)
    // If an org statement does appear before any code, this entry will be removed.
    org_to_code_pos.insert(0, 0);

    // First parser loop. Tokenizes source lines and collects labels.
    *line_num = 0;
    for line in assembly.lines() {
        *line_num += 1;
        let tokenized_line = tokenize(line)?;
        match tokenized_line {
            SourceLine::Blank => (),
            SourceLine::Org(o) => {
                if (o as usize) < code_addr {
                    return Err("org smaller than code address".to_string());
                }

                // If org appears before any code, remove the default, initial org.
                if code_pos == 0 {
                    org_to_code_pos.clear();
                }

                org_to_code_pos.insert(o, code_pos);
                code_addr = o as usize;
            }
            SourceLine::Label(ref s, u) => {
                if labels.contains_key(s) {
                    return Err("label repeated".to_string());
                }
                labels.insert(s.to_string(), u);
            }
            SourceLine::ZByte(ref s, size) => {
                if labels.contains_key(s) {
                    return Err("label repeated".to_string());
                }
                labels.insert(s.to_string(), UInt::U8(config.zpm.alloc(size)));
            }
            SourceLine::Data(ref d) => {
                // Assume labels are two bytes, which is verified later in the second loop.
                let mut data_size: usize = 2;
                if let Rawdata::Bytes(b) = d {
                    data_size = b.len();
                }

                code_addr += data_size;
                code_pos += data_size;
            }
            SourceLine::CodeMarker(ref s) => {
                if labels.contains_key(s) {
                    return Err("label repeated".to_string());
                }
                labels.insert(s.to_string(), UInt::U16(code_addr as u16));
            }
            SourceLine::Instr(ref mnemonic, _, _) => {
                code_addr += get_instr_size(mnemonic)? as usize;
                code_pos += get_instr_size(mnemonic)? as usize;
            }
        }

        // Store all source lines so that next loop can refer to input
        // by line number.
        source.push(tokenized_line);
    }

    // Second parser loop. Stores machine code in "disassembly" vector.
    code_addr = 0;
    *line_num = 0;
    let mut disassembly: Vec<u8> = Vec::new();
    for s in source {
        *line_num += 1;
        match s {
            SourceLine::Org(o) => {
                code_addr = o as usize;
            }
            SourceLine::Data(d) => match d {
                Rawdata::Label(l) => match labels.get(&l) {
                    Some(UInt::U8(_)) => {
                        return Err("labels used for data must be two bytes".to_string())
                    }
                    Some(UInt::U16(u)) => {
                        let bytes = (*u).to_le_bytes();
                        disassembly.push(bytes[0]);
                        disassembly.push(bytes[1]);
                    }
                    None => panic!("Internal error: label {l} found in second pass but not first"),
                },
                Rawdata::Bytes(b) => disassembly.extend(b),
            },
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
                            return Err("offset must be a single byte".to_string())
                        }
                        None => panic!("Internal error: label {l} found in second pass but not first"),
                    },
                }

                // Handle labelled op. Unwrap it and convert it to a non-label variant.
                let input_op_unwrapped: Op;
                if let Op::Label(l) = input_op {
                    input_op_unwrapped = match labels.get(&l) {
                        Some(u) => Op::UInt(*u),
                        None => panic!("Internal error: label {l} found in second pass but not first"),
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
                            return Err("instruction requires a single-byte operand".to_string())
                        }
                        OpType::U16 => {
                            return Err("instruction requires a two-byte operand".to_string())
                        }
                    },

                    // UInt op provided (recall that labels have already been unwrapped)
                    Op::UInt(ui_type) => match ui_type {
                        // UInt op is a single byte
                        UInt::U8(u) => match instr_info.op {
                            OpType::None => {
                                return Err("instruction does not require an operand".to_string())
                            }
                            OpType::U8 => {
                                if u as u16 + offset as u16 > 0xff {
                                    return Err("operand plus offset is > 0xff".to_string());
                                } else {
                                    disassembly.push(u + offset);
                                    code_addr += 1;
                                }
                            }
                            OpType::U16 => {
                                return Err("instruction requires a two-byte operand".to_string())
                            }
                        },

                        // UInt op is two bytes
                        UInt::U16(u) => match instr_info.op {
                            OpType::None => {
                                return Err("instruction does not require an operand".to_string())
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
                                        return Err("operand plus offset is > 0xffff".to_string());
                                    } else {
                                        // Jump is from the end of the current instruction
                                        // (code_addr + 1)
                                        match compute_diff_u16_as_u8(
                                            u + offset as u16,
                                            (code_addr + 1) as u16,
                                        ) {
                                            Some(d) => {
                                                disassembly.push(d);
                                                code_addr += 1;
                                            }
                                            None => {
                                                return Err(
                                                    "relative branch is too far from target"
                                                        .to_string(),
                                                )
                                            }
                                        }
                                    }
                                } else {
                                    return Err(
                                        "instruction requires a single-byte operand".to_string()
                                    );
                                }
                            }
                            OpType::U16 => {
                                if u as u32 + offset as u32 > 0xffff {
                                    return Err("operand plus offset is > 0xffff".to_string());
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

    // Create and write the final output
    let code = bytes_to_output(&disassembly, org_to_code_pos, config.cformat);
    write_code(&code, &config.otype)?;

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
    fn hex3_to_u16() {
        match hex_to_uint("D80") {
            Ok(UInt::U16(i)) => assert_eq!(i, 3456),
            _ => panic!("Unable to convert 2-byte hex to int"),
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
    fn non_hex_to_err() {
        let e = hex_to_uint("John");
        assert!(e.is_err());
    }
}
