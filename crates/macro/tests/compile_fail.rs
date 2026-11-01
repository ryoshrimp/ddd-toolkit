#[test]
#[ignore = "UI suite: run scripts/check-ui-tests.sh"]
fn diagnostics() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
    tests.pass("tests/ui/pass/*.rs");
}
