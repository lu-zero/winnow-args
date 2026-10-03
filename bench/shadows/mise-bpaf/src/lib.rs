//! usage's mise shadow, vendored unmodified in `shadow.rs`.

// Its doc comments are mise's help text: left out when rustdoc collects
// doctests, which would compile every indented example in them.
#[cfg(not(doctest))]
mod shadow;
#[cfg(not(doctest))]
pub use shadow::*;
