use std::collections::BTreeMap;
use std::io::Read;

use crate::config::*;
use crate::output::*;

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

    let mut org_to_code_pos = BTreeMap::new();
    org_to_code_pos.insert(0, 0);
    if let Code::String(s) = bytes_to_output(&bytes, org_to_code_pos, CodeFormat::Hex) {
        println!("{s}");
    }

    Ok(Code::String("".to_string()))
}
