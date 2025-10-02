#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/custom_state.rs");
    t.compile_fail("tests/ui/custom_state_derive_error.rs");

    // Doc comment propagation tests
    t.pass("tests/ui/doc_comments.rs");
    t.compile_fail("tests/ui/doc_comments_missing_lint.rs");
}
