use sasm2;

// Create a string of s repeated n times for creating blocks of repeated code.
fn build_rep_string(s: &str, n: usize) -> String {
    return std::iter::repeat(s).take(n).collect::<String>();
}

// Boilerplate for running an integration test
fn run_string_test(assembly: &str, should_pass: bool, output: &str) {
    let mut c = sasm2::Config::build_string_test(assembly);
    let result = sasm2::run(&mut c);

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
    run_string_test(
        "zbyte z cafe",
        false,
        "0: zbyte array size must be a single byte (< 0x100)",
    );
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
    run_string_test(
        "xxx .op cafe",
        false,
        "0: offset must be a single byte (< 0x100)",
    );
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
    run_string_test(
        "ldyi cafe",
        false,
        "Instruction requires a single-byte operand",
    );
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
fn program_1_simple_instructions() {
    let assembly = "ldaz  ff\n\
                    ldxi  00\n\
                    clc\n\
                    adcax 4000\n\
                    inx\n\
                    adcax 4000\n\
                    staa  6000\n";

    let disassembly = "a5ff\
                       a200\
                       18\
                       7d0040\
                       e8\
                       7d0040\
                       8d0060";

    run_string_test(assembly, true, disassembly);
}

#[test]
fn program_1_with_labels() {
    let assembly = "zbyte z0
                    ldaz  .z0\n\
                    ldxi  00\n\
                    clc\n\
                    label arr1 4000
                    adcax .arr1\n\
                    inx\n\
                    adcax 4000\n\
                    label arr2 5f50
                    label arr2_offset b0
                    staa  .arr2 .arr2_offset\n";

    let disassembly = "a5ff\
                       a200\
                       18\
                       7d0040\
                       e8\
                       7d0040\
                       8d0060";

    run_string_test(assembly, true, disassembly);
}

#[test]
fn rel_branch_backward_barely_in_range() {
    let assembly = ["ldxi  00\n\
                     .loop_start\n\
                     inx\n",
                    &build_rep_string("nop\n", 125),
                    "beq   .loop_start\n"].join("");

    let disassembly = ["a200\
                        e8",
                       &build_rep_string("ea", 125),
                       "f080"].join("");

    run_string_test(&assembly, true, &disassembly);
}

#[test]
fn rel_branch_backward_barely_out_of_range() {
    let assembly = ["ldxi  00\n\
                     .loop_start\n\
                     inx\n",
                    &build_rep_string("nop\n", 126),
                    "beq   .loop_start\n"].join("");

    run_string_test(&assembly, false, "Relative branch is too far from target");
}

#[test]
fn rel_branch_forward_barely_in_range() {
    let assembly = ["ldxi  00\n\
                     .loop_start\n\
                     inx\n\
                     beq   .loop_end\n",
                    &build_rep_string("nop\n", 124),
                    "jmpa  .loop_start\n\
                     .loop_end\n"].join("");

    let disassembly = ["a200\
                        e8\
                        f07f",
                       &build_rep_string("ea", 124),
                       "4c0200"].join("");

    run_string_test(&assembly, true, &disassembly);
}

#[test]
fn rel_branch_forward_barely_out_of_range() {
    let assembly = ["ldxi  00\n\
                     .loop_start\n\
                     inx\n\
                     beq   .loop_end\n",
                    &build_rep_string("nop\n", 125),
                    "jmpa  .loop_start\n\
                     .loop_end\n"].join("");

    run_string_test(&assembly, false, "Relative branch is too far from target");
}
