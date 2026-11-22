#[test]
#[ignore = "UI suite: run scripts/check-ui-tests.sh"]
fn secret_does_not_deref() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/secret_no_deref.rs");
}

#[test]
#[ignore = "UI suite: run scripts/check-ui-tests.sh"]
fn secret_does_not_clone() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/secret_no_clone.rs");
}

#[test]
#[ignore = "UI suite: run scripts/check-ui-tests.sh"]
fn secret_does_not_implement_eq() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/secret_no_eq.rs");
}
