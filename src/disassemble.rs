use std::collections::BTreeSet;
use std::io::Read;

use crate::config::*;
use crate::data::*;
use crate::output::*;

// Maps bytes to their instruction sizes
// Either 1-3 or 0 if byte is not a legal instruction
fn get_instr_sizes_for_bytes(bytes: &Vec<u8>) -> Vec<u8> {
    let mut byte_to_instr_size = vec![0; bytes.len()];
    for i in 0..bytes.len() {
        if let Some(s) = get_instr_size_from_opcode(bytes[i]) {
            byte_to_instr_size[i] = s;
        }
    }
    byte_to_instr_size
}

fn get_code_regions(instr_sizes: &Vec<u8>) -> Vec<(usize, usize)> {
    const MIN_REGION_SIZE: usize = 10;
    let mut regions = Vec::new();

    // Compute possible code region starting from each byte
    for start_pos in 0..instr_sizes.len() {
        let mut end_pos = start_pos;

        // Compute length of code region
        while end_pos < instr_sizes.len() {
            if instr_sizes[end_pos] == 0 {
                break;
            }
            end_pos += instr_sizes[end_pos] as usize;
        }

        if end_pos - start_pos > MIN_REGION_SIZE {
            regions.push((start_pos, end_pos));
        }
    }

    // Sort regions from largest to smallest
    regions.sort_by(|a, b| (b.1 - b.0).cmp(&(a.1 - a.0)));

    // Helper function
    fn regions_overlap(r1: (usize, usize), r2: (usize, usize)) -> bool {
        if r1.1 < r2.0 {
            return false;
        } else if r2.1 < r1.0 {
            return false;
        } else {
            return true;
        }
    }

    // Only keep regions that do not overlap a larger region
    let mut num_selected_regions = 0;
    for i in 0..regions.len() {
        let mut keep_region = true;

        for j in 0..num_selected_regions {
            if regions_overlap(regions[i], regions[j]) {
                keep_region = false;
                break;
            }
        }

        if keep_region {
            regions[num_selected_regions] = regions[i];
            num_selected_regions += 1;
        }
    }

    // Remove non-selected regions and sort by starting value
    regions.resize(num_selected_regions, (0, 0));
    regions.sort_by(|a, b| a.0.cmp(&b.0));

    return regions;
}

fn get_assembly_from_bytes(
    bytes: &Vec<u8>,
    regions: &Vec<(usize, usize)>,
    start_addr: u16,
) -> Code {
    struct SourceLine(u16, String);

    // First disassembly loop. This loop does the majority of the work, creating the output source
    // lines (minus labels) and also finding and storing labels.
    let mut last_region_end_byte = 0;
    let mut source = Vec::new();
    let mut labeled_addrs = BTreeSet::new();

    for (start_byte_ref, end_byte_ref) in regions {
        let start_byte = *start_byte_ref;
        let end_byte = *end_byte_ref;

        // Write data before region
        if last_region_end_byte < start_byte {
            let hex = hex::encode(&bytes[last_region_end_byte..start_byte]);
            source.push(SourceLine(
                last_region_end_byte as u16 + start_addr,
                format!("data  {hex}"),
            ));
        }

        // Write code in region
        let err_string = "Internal error: found invalid opcode while creating assembly";
        let mut current_byte = start_byte;
        while current_byte < end_byte {
            let instr_info = get_instr_info_from_opcode(bytes[current_byte]).expect(err_string);
            let instr_size: usize = get_instr_size_from_opcode(bytes[current_byte])
                .expect(err_string)
                .into();
            let mnemonic = &instr_info.mnemonic;
            let padding = " ".repeat(6 - mnemonic.len());

            // Write a single instruction

            // Case 1: instruction has an address, so we need to use a label
            if instr_size > 2 || is_relative_branch_instruction(&instr_info.mnemonic) {
                let new_addr = match instr_size {
                    2 => {
                        // relative address
                        let instr_addr = start_addr as usize + current_byte + instr_size;
                        let abs_addr = instr_addr as isize + bytes[current_byte + 1] as i8 as isize;
                        assert!(abs_addr >= 0, "Error: relative address has absolute address less than 0");
                        abs_addr as usize
                    }
                    3 => {
                        // absolute address
                        bytes[current_byte + 2] as usize * 256 + bytes[current_byte + 1] as usize
                    }
                    _ => panic!("Internal error: impossible size for branch instruction"),
                };

                // Do not use a label for addresses outside the program's address space
                // Currently, the label is the address prepended with a dot, so just remove the
                // dot to insert the explicit address.
                let mut optional_dot = ".";
                if new_addr < start_addr as usize || new_addr >= start_addr as usize + bytes.len() {
                    optional_dot = "";
                } else if !labeled_addrs.contains(&new_addr) {
                    labeled_addrs.insert(new_addr);
                }
                source.push(SourceLine(
                    current_byte as u16 + start_addr,
                    format!("{mnemonic}{padding}{optional_dot}{:04x}", new_addr),
                ));

            // Case 2: instruction has a single operand that is not an address
            } else if instr_size > 1 {
                source.push(SourceLine(
                    current_byte as u16 + start_addr,
                    format!("{mnemonic}{padding}{:02x}", bytes[current_byte + 1]),
                ));

            // Case 3: instruction has no operands
            } else {
                source.push(SourceLine(
                    current_byte as u16 + start_addr,
                    format!("{mnemonic}"),
                ));
            }

            current_byte += instr_size;
        }

        last_region_end_byte = end_byte;
    }

    // Write data after last region
    if last_region_end_byte < bytes.len() {
        let hex = hex::encode(&bytes[last_region_end_byte..bytes.len()]);
        source.push(SourceLine(
            last_region_end_byte as u16 + start_addr,
            format!("data  {hex}"),
        ));
    }

    // Second disassembly loop. Join source lines, inserting labels at the proper locations.
    let mut assembly = String::new();
    let mut current_line = 1;

    // Labeled addresses are sorted. Add a sentinel value to avoid handling NONEs
    let addr_error = "Internal error: ran out of labeled addresses";
    labeled_addrs.insert(0x10000);
    let mut labeled_addr_iter = labeled_addrs.iter();
    let mut next_labeled_addr = *labeled_addr_iter.next().expect(addr_error);

    // First line is the starting address
    assembly.push_str(&format!("org   {:04x}\n", start_addr));
    current_line += 1;

    for s in source {
        // Watch out for labels not on an instruction or data section boundary
        while s.0 as usize > next_labeled_addr {
            eprintln!("Warning: address {:04x} inside line {}", next_labeled_addr, current_line - 1);
            next_labeled_addr = *labeled_addr_iter.next().expect(addr_error);
        }

        // Insert label
        if s.0 as usize == next_labeled_addr {
            assembly.push_str(&format!(".{:04x}\n", s.0));
            current_line += 1;
            next_labeled_addr = *labeled_addr_iter.next().expect(addr_error);
        }

        // Insert source line
        assembly.push_str(&s.1);
        assembly.push_str("\n");
        current_line += 1;
    }

    Code::String(assembly)
}

pub fn disassemble(config: &mut Config) -> Result<Code, String> {
    let bytes = match config.itype {
        IType::Stdin => {
            let mut b: Vec<u8> = Vec::new();
            match std::io::stdin().read_to_end(&mut b) {
                Ok(_) => b,
                Err(_) => return Err("Unable to read from stdin".to_string()),
            }
        }

        IType::String(ref s) => match hex::decode(s) {
            Ok(b) => b,
            _ => return Err("Cannot decode input string".to_string()),
        },

        IType::File(ref f) => match std::fs::read(f) {
            Ok(b) => b,
            Err(_) => return Err("Unable to read input file".to_string()),
        },
    };

    let bytes_to_instr_size = get_instr_sizes_for_bytes(&bytes);
    let code_regions = get_code_regions(&bytes_to_instr_size);
    let assembly = get_assembly_from_bytes(&bytes, &code_regions, config.addr);
    write_code(&assembly, &config.otype)?;

    Ok(assembly)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_bytes_to_instr_sizes() {
        let bytes: Vec<u8> = vec![0x00, 0x22, 0xc0, 0xfe, 0xaa, 0xff];
        let sizes: Vec<u8> = vec![1, 0, 2, 3, 1, 0];
        assert_eq!(get_instr_sizes_for_bytes(&bytes), sizes);
    }
}
