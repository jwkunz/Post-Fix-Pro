//! Crate entrypoint for the Post_Fix_Pro calculator backend.
//!
//! This file intentionally stays thin and only wires modules + public exports.

pub mod api;
pub mod calculator;
pub mod script;
pub mod types;

/// Stateful calculator engine and operation implementations.
pub use calculator::Calculator;
/// Core value/types used by the calculator and API.
pub use types::{AngleMode, CalcError, CalcState, Complex, DisplayMode, Matrix, Value};

#[cfg(test)]
#[path = "tests/lib_tests.rs"]
mod tests;
