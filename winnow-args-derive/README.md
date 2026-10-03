# winnow-args-derive

[![LICENSE](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/lu-zero/winnow-args#license)
[![Crates.io](https://img.shields.io/crates/v/winnow-args-derive.svg)](https://crates.io/crates/winnow-args-derive)
[![docs.rs](https://docs.rs/winnow-args-derive/badge.svg)](https://docs.rs/winnow-args-derive)
[![CI](https://github.com/lu-zero/winnow-args/actions/workflows/ci.yml/badge.svg)](https://github.com/lu-zero/winnow-args/actions/workflows/ci.yml)

The derive macros of [winnow-args](https://crates.io/crates/winnow-args):
`Args`, `Subcommand`, `ValueEnum` and `Occurrence`.

Depend on `winnow-args`, which re-exports them under its `derive` feature (on
by default); this crate is not meant to be used on its own. The attributes
each derive accepts are listed in its documentation.
