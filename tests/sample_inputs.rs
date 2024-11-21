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
