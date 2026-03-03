pub mod api;
pub mod calculator;
pub mod types;

pub use calculator::Calculator;
pub use types::{AngleMode, CalcError, CalcState, Complex, DisplayMode, Matrix, Value};

#[cfg(test)]
#[path = "tests/lib_tests.rs"]
mod tests;
