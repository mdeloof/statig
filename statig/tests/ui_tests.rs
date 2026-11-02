#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/custom_state.rs");
    t.compile_fail("tests/ui/custom_state_derive_error.rs");

    // Doc comment propagation tests
    t.pass("tests/ui/doc_comments.rs");
    t.compile_fail("tests/ui/doc_comments_missing_lint.rs");

    // Context type declaration tests
    t.compile_fail("tests/ui/context_attribute_missing.rs");
    t.compile_fail("tests/ui/context_attribute_missing_order.rs");
    t.compile_fail("tests/ui/context_attribute_missing_superstate.rs");
    t.compile_fail("tests/ui/context_attribute_duplicate.rs");
    t.compile_fail("tests/ui/context_attribute_not_a_type.rs");

    // Context type declarations that disagree with the handlers
    t.compile_fail("tests/ui/context_attribute_mismatch.rs");
    t.compile_fail("tests/ui/context_attribute_foreign_lifetime.rs");
}
