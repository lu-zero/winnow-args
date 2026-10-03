//! The derive's compile errors, pinned: `tests/ui/*.stderr` is what each case
//! must report. `TRYBUILD=overwrite cargo test compile_fail` rewrites
//! them after a deliberate change.

#[test]
fn derive_errors() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
