//! Test support shared by the VICE boot targets (`boot.rs`, `menu_boot.rs`).
//!
//! Each target compiles this module on its own and uses only part of it, so unused items are
//! expected here.
#![allow(dead_code)]

pub mod markers;
pub mod vice;
