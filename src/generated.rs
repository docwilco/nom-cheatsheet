// String literals in formats are inefficient, but this saves us from having to
// escape braces in a couple of places in build.rs
#![allow(clippy::write_literal)]

include!(concat!(env!("OUT_DIR"), "/generated.rs"));
