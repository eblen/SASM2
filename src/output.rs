use std::collections::BTreeMap;

pub fn hex_format(bytes: &Vec<u8>, org_to_code_pos: BTreeMap<u16, u16>) -> String {
    let filler_byte = "ff";
    let mut output = String::new();

    // Convert values to usize for array indexing
    let mut org_iter = org_to_code_pos
        .iter()
        .map(|x| (*x.0 as usize, *x.1 as usize));

    // Get first org
    let (mut prev_org, mut prev_pos) = org_iter
        .next()
        .expect("Internal error: no org found for assembled code");

    for (org, pos) in org_iter {
        // Create hex for bytes between the previous and current orgs
        output.push_str(&hex::encode(&bytes[prev_pos..pos]));

        // Fill remaining space with the filler hex value
        let gap_size = org - prev_org - (pos - prev_pos);
        output.push_str(
            &std::iter::repeat(filler_byte)
                .take(gap_size)
                .collect::<String>(),
        );

        prev_org = org;
        prev_pos = pos;
    }

    // Create hex for bytes after the last org
    output.push_str(&hex::encode(&bytes[prev_pos..]));

    output
}
