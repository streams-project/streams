//! A response cannot be made without the rules (H34 K3, critic's rule 4).

#[test]
fn h34_k3_safe_types_cannot_be_built_outside_the_rules() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/compile_fail/*.rs");
}
