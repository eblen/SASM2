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

fn get_assembly_from_bytes(bytes: &Vec<u8>, regions: &Vec<(usize, usize)>) -> Code {
    let mut assembly = String::new();
    let mut last_region_end = 0;

    for (start, end) in regions {
        // Write data before region
        if last_region_end < *start {
            assembly.push_str("data  ");
            assembly.push_str(&hex::encode(&bytes[last_region_end..*start]));
            assembly.push_str("\n");
        }

        // Write code in region
        let err_string = "Internal error: found invalid opcode while creating assembly";
        let mut code_pos = *start;
        while code_pos < *end {
            let instr_info = get_instr_info_from_opcode(bytes[code_pos]).expect(err_string);
            let instr_size: usize = get_instr_size_from_opcode(bytes[code_pos]).expect(err_string).into();

            // Write mnemonic
            assembly.push_str(&instr_info.mnemonic);
            assembly.push_str(&" ".repeat(6 - instr_info.mnemonic.len()));

            // Write operand
            // Remember to write two-byte operands in big endian.
            if instr_size > 2 {
                assembly.push_str(&format!("{:x}", bytes[code_pos + 2]));
                // assembly.push_str(&hex::encode(bytes[code_pos + 2]));
            }
            if instr_size > 1 {
                assembly.push_str(&format!("{:02x}", bytes[code_pos + 1]));
                // assembly.push_str(&hex::encode(bytes[code_pos + 1]));
            }
            assembly.push_str("\n");

            code_pos += instr_size;
        }

        last_region_end = *end;
    }

    // Write data after last region
    if last_region_end < bytes.len() {
        assembly.push_str("data  ");
        assembly.push_str(&hex::encode(&bytes[last_region_end..bytes.len()]));
        assembly.push_str("\n");
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
    let assembly = get_assembly_from_bytes(&bytes, &code_regions);
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
