use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub enum CodeFormat {
    // String of hex digits
    Hex,

    // Apple II system monitor
    AppleSM,

    // Binary code
    Binary,
}

#[derive(Debug, PartialEq)]
pub enum Code {
    // For Hex and AppleSM formats
    String(String),

    // For Binary format
    Bytes(Vec<u8>),
}

impl CodeFormat {
    // Attempt to create a variant from a string.
    // Since first letters are currently all unique, just rely on them for now.
    pub fn new(format: &str) -> Result<Self, &str> {
        match format
            .to_ascii_lowercase()
            .chars()
            .next()
            .expect("Internal error: Empty CLI argument")
        {
            'h' => Ok(CodeFormat::Hex),
            'a' => Ok(CodeFormat::AppleSM),
            'b' => Ok(CodeFormat::Binary),
            _ => Err("Unrecognized code format"),
        }
    }

    fn code_for_org_block(&self, start_addr: usize, end_addr: usize, bytes: &[u8]) -> Code {
        match self {
            CodeFormat::Hex => Self::org_block_for_hex(start_addr, end_addr, bytes),
            CodeFormat::AppleSM => Self::org_block_for_apple_sm(start_addr, end_addr, bytes),
            CodeFormat::Binary => Self::org_block_for_binary(start_addr, end_addr, bytes),
        }
    }

    fn org_block_for_hex(start_addr: usize, end_addr: usize, bytes: &[u8]) -> Code {
        // Encode bytes as a string of hex values
        let mut code_as_string = hex::encode(bytes);

        // Fill remaining space with the filler hex value (0xff)
        let gap_size = end_addr - start_addr - bytes.len();
        code_as_string.push_str(&std::iter::repeat("ff").take(gap_size).collect::<String>());

        return Code::String(code_as_string);
    }

    fn org_block_for_binary(start_addr: usize, end_addr: usize, bytes: &[u8]) -> Code {
        // Nothing to do for code except copy it
        let mut code_as_bytes = bytes.to_vec();

        // Fill remaining space with the filler byte (255)
        let gap_size = end_addr - start_addr - bytes.len();
        code_as_bytes.extend(std::iter::repeat(255).take(gap_size));

        return Code::Bytes(code_as_bytes);
    }

    // todo!
    fn org_block_for_apple_sm(_start_addr: usize, _end_addr: usize, _bytes: &[u8]) -> Code {
        return Code::String("".to_string());
    }
}

// Convert assembled bytes to the proper output format (a string to be printed)
// This function iterates through pairs of orgs, while the format-specific code resides in
// separate functions.
pub fn bytes_to_output(
    bytes: &[u8],
    org_to_code_pos: BTreeMap<u16, u16>,
    format: CodeFormat,
) -> Code {
    let mut org_blocks = Vec::new();

    // Convert values to usize for array indexing
    let mut org_iter = org_to_code_pos
        .iter()
        .map(|x| (*x.0 as usize, *x.1 as usize));

    // Get first org
    let (mut prev_org, mut prev_pos) = org_iter
        .next()
        .expect("Internal error: no org found for assembled code");

    for (org, pos) in org_iter {
        // Generate code blocks between orgs
        org_blocks.push(format.code_for_org_block(prev_org, org, &bytes[prev_pos..pos]));

        prev_org = org;
        prev_pos = pos;
    }

    // Generate code block after last org.
    // Length is the size of the remaining bytes to ensure no filler bytes are printed.
    let end_org = prev_org + bytes.len() - prev_pos;
    org_blocks.push(format.code_for_org_block(prev_org, end_org, &bytes[prev_pos..]));

    // Join org blocks
    match format {
        CodeFormat::Hex | CodeFormat::AppleSM => {
            let code_as_string = org_blocks
                .iter()
                .fold(String::new(), |code, block| match block {
                    Code::String(s) => code + &s,
                    _ => panic!("Internal error: wrong output type encountered"),
                });
            return Code::String(code_as_string);
        }
        CodeFormat::Binary => {
            let code_as_bytes = org_blocks
                .iter()
                .fold(Vec::new(), |mut code, block| match block {
                    Code::Bytes(b) => {
                        code.extend(b);
                        code
                    }
                    _ => panic!("Internal error: wrong output type encountered"),
                });
            return Code::Bytes(code_as_bytes);
        }
    }
}
