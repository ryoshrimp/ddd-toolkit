#[cfg(feature = "serde")]
#[test]
#[ignore = "UI suite: run scripts/check-ui-tests.sh"]
fn secret_does_not_implement_serde() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/secret_serde.rs");
}
