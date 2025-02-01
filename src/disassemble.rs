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

pub fn disassemble(config: &mut Config) -> Result<Code, String> {
    let bytes = match config.itype {
        IType::Stdin => {
            let mut b: Vec<u8> = Vec::new();
            match std::io::stdin().read_to_end(&mut b) {
                Ok(_) => b,
                Err(_) => return Err("Unable to read from stdin".to_string()),
            }
        }

        IType::String(ref s) => {
            match hex::decode(s) {
                Ok(b) => b,
                _ => return Err("Cannot decode input string".to_string()),
            }
        }

        IType::File(ref f) => match std::fs::read(f) {
            Ok(b) => b,
            Err(_) => return Err("Unable to read input file".to_string()),
        },
    };

    let bytes_to_instr_size = get_instr_sizes_for_bytes(&bytes);
    println!("{:?}", bytes_to_instr_size);

    Ok(Code::String("".to_string()))
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
