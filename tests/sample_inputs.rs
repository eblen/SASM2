use sasm2;

fn run_string_test(assembly: &str, should_pass: bool, output: &str) {
    let c = sasm2::Config::build_string_test(assembly);
    let result = sasm2::run(c);

    if should_pass {
        assert_eq!(result, Ok(output.to_string()));
    } else {
        assert_eq!(result, Err(output.to_string()));
    }
}

// Tests Tokenizer
#[test]
fn org_address_too_small() {
    run_string_test("org 88", false, "0: org must be a 2-byte address");
}

#[test]
fn org_address_missing() {
    run_string_test("org", false, "0: org takes one argument");
}

#[test]
fn org_address_fine() {
    run_string_test("org ABCD", true, "");
}

#[test]
fn data_forward() {
    run_string_test("data CaFe", true, "cafe");
}

#[test]
fn data_odd_size() {
    run_string_test("data cafedad", false, "0: data must be a valid hex string");
}

#[test]
fn data_non_hex() {
    run_string_test("data coffee", false, "0: data must be a valid hex string");
}

#[test]
fn data_with_spaces() {
    run_string_test("data cafe dad", false, "0: data takes one argument");
}

#[test]
fn zbyte_size_too_big() {
    run_string_test("zbyte z cafe", false, "0: zbyte array size must be a single byte (< 0x100)");
}

#[test]
fn zbyte_odd_size() {
    run_string_test("zbyte z dad", false, "0: not a valid hexadecimal number");
}

#[test]
fn zbyte_non_hex() {
    run_string_test("zbyte z pa", false, "0: not a valid hexadecimal number");
}

#[test]
fn label_odd_size() {
    run_string_test("label l dad", false, "0: not a valid hexadecimal number");
}

#[test]
fn label_non_hex() {
    run_string_test("label l pa", false, "0: not a valid hexadecimal number");
}

#[test]
fn instr_op_odd_size() {
    run_string_test("xxx dad", false, "0: not a valid hexadecimal number");
}

#[test]
fn instr_op_non_hex() {
    run_string_test("xxx john", false, "0: not a valid hexadecimal number");
}

#[test]
fn instr_offset_too_big() {
    run_string_test("xxx .op cafe", false, "0: offset must be a single byte (< 0x100)");
}

#[test]
fn instr_offset_odd_size() {
    run_string_test("xxx ff dad", false, "0: not a valid hexadecimal number");
}

#[test]
fn instr_offset_non_hex() {
    run_string_test("xxx ff john", false, "0: not a valid hexadecimal number");
}

// Tests Parser
#[test]
fn bad_instr() {
    run_string_test("dec", false, "Mnemonic not found");
}

#[test]
fn needs_u8_op() {
    run_string_test("andz", false, "Instruction requires a single-byte operand");
}

#[test]
fn needs_u16_op() {
    run_string_test("adcax", false, "Instruction requires a two-byte operand");
}

#[test]
fn op_too_small() {
    run_string_test("oraa ff", false, "Instruction requires a two-byte operand");
}

#[test]
fn op_too_big() {
    run_string_test("ldyi cafe", false, "Instruction requires a single-byte operand");
}

#[test]
fn u8_op_not_needed() {
    run_string_test("clc ff", false, "Instruction does not require an operand");
}

#[test]
fn u16_op_not_needed() {
    run_string_test("dex ffff", false, "Instruction does not require an operand");
}

#[test]
fn u8_offset_too_big() {
    run_string_test("staz fe 2", false, "Operand plus offset is > 0xff");
}

#[test]
fn u16_offset_too_big() {
    run_string_test("staa fffe 2", false, "Operand plus offset is > 0xffff");
}

#[test]
fn program_with_no_labels() {
    let assembly = "ldaz  44\n\
                    ldxi  00\n\
                    clc\n\
                    adcax 4000\n\
                    inx\n\
                    adcax 4000\n\
                    staa  6000\n";

    let disassembly = "a544\
                       a200\
                       18\
                       7d0040\
                       e8\
                       7d0040\
                       8d0060";

    run_string_test(assembly, true, disassembly);
}
