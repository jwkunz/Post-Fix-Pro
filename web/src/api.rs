//! Serializable API surface for driving the calculator from JavaScript/WASM.

use serde::{Deserialize, Serialize};

use crate::script::{
    ScriptResponse, run_script_line_text, run_script_text,
};
use crate::{AngleMode, CalcError, CalcState, Calculator, Complex, DisplayMode, Matrix, Value};

/// Serialized value payload used by API requests/responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiValue {
    /// Real scalar value.
    Real {
        /// Numeric value.
        value: f64,
    },
    /// Complex scalar value.
    Complex {
        /// Real component.
        re: f64,
        /// Imaginary component.
        im: f64,
    },
    /// Dense matrix value with row-major complex data.
    Matrix {
        /// Number of rows.
        rows: usize,
        /// Number of columns.
        cols: usize,
        /// Row-major entries.
        data: Vec<ComplexInput>,
    },
}

/// API angle mode payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiAngleMode {
    Deg,
    Rad,
}

/// API display mode payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiDisplayMode {
    Fix,
    Sci,
    Eng,
}

/// Snapshot of full calculator state returned to the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiState {
    /// Bottom-to-top stack values.
    pub stack: Vec<ApiValue>,
    /// Current scalar entry buffer text.
    pub entry_buffer: String,
    /// Active angle mode.
    pub angle_mode: ApiAngleMode,
    /// Active display mode.
    pub display_mode: ApiDisplayMode,
    /// Decimal precision.
    pub precision: u8,
    /// Memory registers.
    pub memory: Vec<Option<ApiValue>>,
}

/// Structured error payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

/// Standard response envelope for API operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Whether the command succeeded.
    pub ok: bool,
    /// Updated calculator state.
    pub state: ApiState,
    /// Error details when `ok` is false.
    pub error: Option<ApiError>,
    /// Optional non-fatal warning.
    pub warning: Option<String>,
}

/// Matrix input payload accepted by `push_matrix`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixInput {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub cols: usize,
    /// Row-major matrix entries.
    pub data: Vec<ComplexInput>,
}

/// Complex input payload shared by scalar and matrix APIs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexInput {
    /// Real component.
    pub re: f64,
    /// Imaginary component.
    pub im: f64,
}

/// High-level API wrapper over [`Calculator`] for command-style consumption.
#[derive(Debug, Clone, PartialEq)]
pub struct CalculatorApi {
    calculator: Calculator,
}

impl Default for CalculatorApi {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorApi {
    /// Constructs a new instance.
    pub fn new() -> Self {
        Self {
            calculator: Calculator::new(),
        }
    }

    /// Returns a serializable snapshot of the current state.
    pub fn snapshot(&self) -> ApiState {
        to_api_state(self.calculator.state())
    }

    /// Executes the `entry_set` operation.
    pub fn entry_set(&mut self, value: &str) -> ApiResponse {
        let before = self.calculator.state().clone();
        self.calculator.entry_set(value);
        self.success_with_undo(before)
    }

    /// Executes the `clear_entry` operation.
    pub fn clear_entry(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        self.calculator.clear_entry();
        self.success_with_undo(before)
    }

    /// Executes the `clear_all` operation.
    pub fn clear_all(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        self.calculator.clear_all();
        self.success_with_undo(before)
    }

    /// Executes the `push_real` operation.
    pub fn push_real(&mut self, value: f64) -> ApiResponse {
        let before = self.calculator.state().clone();
        self.calculator.push_value(Value::Real(value));
        self.success_with_undo(before)
    }

    /// Executes the `push_complex` operation.
    pub fn push_complex(&mut self, complex: ComplexInput) -> ApiResponse {
        let before = self.calculator.state().clone();
        self.calculator.push_value(Value::Complex(Complex {
            re: complex.re,
            im: complex.im,
        }));
        self.success_with_undo(before)
    }

    /// Executes the `push_matrix` operation.
    pub fn push_matrix(&mut self, matrix: MatrixInput) -> ApiResponse {
        let data = matrix
            .data
            .into_iter()
            .map(|entry| Complex {
                re: entry.re,
                im: entry.im,
            })
            .collect::<Vec<_>>();
        match Matrix::new(matrix.rows, matrix.cols, data) {
            Ok(value) => {
                let before = self.calculator.state().clone();
                self.calculator.push_value(Value::Matrix(value));
                self.success_with_undo(before)
            }
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
                warning: None,
            },
        }
    }

    /// Executes the `set_angle_mode` operation.
    pub fn set_angle_mode(&mut self, mode: ApiAngleMode) -> ApiResponse {
        let before = self.calculator.state().clone();
        let mode = match mode {
            ApiAngleMode::Deg => AngleMode::Deg,
            ApiAngleMode::Rad => AngleMode::Rad,
        };
        self.calculator.set_angle_mode(mode);
        self.success_with_undo(before)
    }

    /// Executes the `set_display_mode` operation.
    pub fn set_display_mode(&mut self, mode: ApiDisplayMode) -> ApiResponse {
        let before = self.calculator.state().clone();
        let mode = match mode {
            ApiDisplayMode::Fix => DisplayMode::Fix,
            ApiDisplayMode::Sci => DisplayMode::Sci,
            ApiDisplayMode::Eng => DisplayMode::Eng,
        };
        self.calculator.set_display_mode(mode);
        self.success_with_undo(before)
    }

    /// Executes the `set_precision` operation.
    pub fn set_precision(&mut self, precision: u8) -> ApiResponse {
        let before = self.calculator.state().clone();
        self.calculator.set_precision(precision);
        self.success_with_undo(before)
    }

    /// Executes the `enter` operation.
    pub fn enter(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.enter();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `drop` operation.
    pub fn drop(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.drop().map(|_| ());
        self.wrap_with_undo(before, result)
    }

    /// Executes the `dup` operation.
    pub fn dup(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.dup();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `swap` operation.
    pub fn swap(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.swap();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `rot` operation.
    pub fn rot(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.rot();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `roll` operation.
    pub fn roll(&mut self, count: usize) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.roll(count);
        self.wrap_with_undo(before, result)
    }

    /// Executes the `pick` operation.
    pub fn pick(&mut self, depth: usize) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.pick(depth);
        self.wrap_with_undo(before, result)
    }

    /// Executes the `pick_from_stack_index` operation.
    pub fn pick_from_stack_index(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.pick_from_stack_index();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `pow` operation.
    pub fn pow(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.pow();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `percent` operation.
    pub fn percent(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.percent();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `inv` operation.
    pub fn inv(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.inv();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `square` operation.
    pub fn square(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.square();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `root` operation.
    pub fn root(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.root();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `add` operation.
    pub fn add(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.add();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `sub` operation.
    pub fn sub(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.sub();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `mul` operation.
    pub fn mul(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.mul();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `div` operation.
    pub fn div(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.div();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `hadamard_mul` operation.
    pub fn hadamard_mul(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.hadamard_mul();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `hadamard_div` operation.
    pub fn hadamard_div(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.hadamard_div();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `sqrt` operation.
    pub fn sqrt(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.sqrt();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `ln` operation.
    pub fn ln(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.ln();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `sin` operation.
    pub fn sin(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.sin();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `cos` operation.
    pub fn cos(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.cos();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `tan` operation.
    pub fn tan(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.tan();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `sec` operation.
    pub fn sec(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.sec();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `csc` operation.
    pub fn csc(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.csc();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `cot` operation.
    pub fn cot(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.cot();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `asin` operation.
    pub fn asin(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.asin();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `acos` operation.
    pub fn acos(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.acos();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `asec` operation.
    pub fn asec(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.asec();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `acsc` operation.
    pub fn acsc(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.acsc();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `atan` operation.
    pub fn atan(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.atan();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `acot` operation.
    pub fn acot(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.acot();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `sinh` operation.
    pub fn sinh(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.sinh();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `cosh` operation.
    pub fn cosh(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.cosh();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `tanh` operation.
    pub fn tanh(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.tanh();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `sech` operation.
    pub fn sech(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.sech();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `csch` operation.
    pub fn csch(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.csch();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `coth` operation.
    pub fn coth(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.coth();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `asinh` operation.
    pub fn asinh(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.asinh();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `acosh` operation.
    pub fn acosh(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.acosh();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `asech` operation.
    pub fn asech(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.asech();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `acsch` operation.
    pub fn acsch(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.acsch();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `atanh` operation.
    pub fn atanh(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.atanh();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `acoth` operation.
    pub fn acoth(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.acoth();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `exp` operation.
    pub fn exp(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.exp();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `exp10` operation.
    pub fn exp10(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.exp10();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `exp2` operation.
    pub fn exp2(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.exp2();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `log10` operation.
    pub fn log10(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.log10();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `log2` operation.
    pub fn log2(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.log2();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `log_y_x` operation.
    pub fn log_y_x(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.log_y_x();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `gamma` operation.
    pub fn gamma(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.gamma();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `erf` operation.
    pub fn erf(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.erf();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `erfc` operation.
    pub fn erfc(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.erfc();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `bessel` operation.
    pub fn bessel(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.bessel();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `mbessel` operation.
    pub fn mbessel(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.mbessel();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `sinc` operation.
    pub fn sinc(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.sinc();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `neg` operation.
    pub fn neg(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.neg();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `signum` operation.
    pub fn signum(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.signum();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `abs` operation.
    pub fn abs(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.abs();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `abs_sq` operation.
    pub fn abs_sq(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.abs_sq();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `arg` operation.
    pub fn arg(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.arg();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `conjugate` operation.
    pub fn conjugate(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.conjugate();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `real_part` operation.
    pub fn real_part(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.real_part();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `imag_part` operation.
    pub fn imag_part(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.imag_part();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `cart` operation.
    pub fn cart(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.cart();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `pol` operation.
    pub fn pol(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.pol();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `npol` operation.
    pub fn npol(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.npol();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `atan2` operation.
    pub fn atan2(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.atan2();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `to_rad` operation.
    pub fn to_rad(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.to_rad();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `to_deg` operation.
    pub fn to_deg(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.to_deg();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `factorial` operation.
    pub fn factorial(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.factorial();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `ncr` operation.
    pub fn ncr(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.ncr();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `npr` operation.
    pub fn npr(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.npr();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `modulo` operation.
    pub fn modulo(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.modulo();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `rand_num` operation.
    pub fn rand_num(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.rand_num();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `gcd` operation.
    pub fn gcd(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.gcd();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `lcm` operation.
    pub fn lcm(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.lcm();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `round_value` operation.
    pub fn round_value(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.round_value();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `floor_value` operation.
    pub fn floor_value(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.floor_value();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `ceil_value` operation.
    pub fn ceil_value(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.ceil_value();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `dec_part` operation.
    pub fn dec_part(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.dec_part();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `push_pi` operation.
    pub fn push_pi(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        self.calculator.push_pi();
        self.success_with_undo(before)
    }

    /// Executes the `push_e` operation.
    pub fn push_e(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        self.calculator.push_e();
        self.success_with_undo(before)
    }

    /// Executes the `determinant` operation.
    pub fn determinant(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.determinant();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `inverse` operation.
    pub fn inverse(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.inverse();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `transpose` operation.
    pub fn transpose(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.transpose();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `solve_ax_b` operation.
    pub fn solve_ax_b(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.solve_ax_b();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `solve_lstsq` operation.
    pub fn solve_lstsq(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        match self.calculator.solve_lstsq() {
            Ok(warning) => self.success_with_warning_and_undo(before, warning),
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
                warning: None,
            },
        }
    }

    /// Executes the `dot` operation.
    pub fn dot(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.dot();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `cross` operation.
    pub fn cross(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.cross();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `trace` operation.
    pub fn trace(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.trace();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `norm_p` operation.
    pub fn norm_p(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.norm_p();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `diag` operation.
    pub fn diag(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.diag();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `toep` operation.
    pub fn toep(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.toep();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `mat_exp` operation.
    pub fn mat_exp(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.mat_exp();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `hermitian` operation.
    pub fn hermitian(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.hermitian();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `mat_pow` operation.
    pub fn mat_pow(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.mat_pow();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `qr` operation.
    pub fn qr(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.qr();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `lu` operation.
    pub fn lu(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.lu();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `svd` operation.
    pub fn svd(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.svd();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `evd` operation.
    pub fn evd(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        match self.calculator.evd() {
            Ok(warning) => self.success_with_warning_and_undo(before, warning),
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
                warning: None,
            },
        }
    }

    /// Executes the `mean` operation.
    pub fn mean(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.mean();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `mode` operation.
    pub fn mode(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.mode();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `variance` operation.
    pub fn variance(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.variance();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `std_dev_p` operation.
    pub fn std_dev_p(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.std_dev_p();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `std_dev_s` operation.
    pub fn std_dev_s(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.std_dev_s();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `median` operation.
    pub fn median(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.median();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `quart` operation.
    pub fn quart(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.quart();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `max_value` operation.
    pub fn max_value(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.max_value();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `min_value` operation.
    pub fn min_value(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.min_value();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `push_identity` operation.
    pub fn push_identity(&mut self, size: usize) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.push_identity(size);
        self.wrap_with_undo(before, result)
    }

    /// Executes the `stack_vec` operation.
    pub fn stack_vec(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.stack_vec();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `hstack` operation.
    pub fn hstack(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.hstack();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `vstack` operation.
    pub fn vstack(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.vstack();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `ravel` operation.
    pub fn ravel(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.ravel();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `hravel` operation.
    pub fn hravel(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.hravel();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `vravel` operation.
    pub fn vravel(&mut self) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.vravel();
        self.wrap_with_undo(before, result)
    }

    /// Executes the `memory_store` operation.
    pub fn memory_store(&mut self, register: usize) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.memory_store(register);
        self.wrap_with_undo(before, result)
    }

    /// Executes the `memory_recall` operation.
    pub fn memory_recall(&mut self, register: usize) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.memory_recall(register);
        self.wrap_with_undo(before, result)
    }

    /// Executes the `memory_clear` operation.
    pub fn memory_clear(&mut self, register: usize) -> ApiResponse {
        let before = self.calculator.state().clone();
        let result = self.calculator.memory_clear(register);
        self.wrap_with_undo(before, result)
    }

    /// Executes the `undo` operation.
    pub fn undo(&mut self) -> ApiResponse {
        let result = self.calculator.undo();
        self.wrap(result)
    }

    /// Executes a batch script.
    pub fn run_script(&mut self, script_text: &str) -> ScriptResponse {
        let before = self.calculator.state().clone();
        let mut working = self.clone();
        match run_script_text(&mut working, script_text) {
            Ok(outcome) => {
                self.calculator = working.calculator;
                self.calculator.set_undo_state(before);
                ScriptResponse {
                    ok: true,
                    state: self.snapshot(),
                    error: None,
                    warning: outcome.warning,
                    transcript: outcome.transcript,
                }
            }
            Err(failure) => ScriptResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(failure.error),
                warning: None,
                transcript: failure.transcript,
            },
        }
    }

    /// Executes a single interactive script line.
    pub fn run_script_line(&mut self, script_line: &str) -> ScriptResponse {
        let before = self.calculator.state().clone();
        let mut working = self.clone();
        match run_script_line_text(&mut working, script_line) {
            Ok(outcome) => {
                self.calculator = working.calculator;
                self.calculator.set_undo_state(before);
                ScriptResponse {
                    ok: true,
                    state: self.snapshot(),
                    error: None,
                    warning: outcome.warning,
                    transcript: outcome.transcript,
                }
            }
            Err(failure) => ScriptResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(failure.error),
                warning: None,
                transcript: failure.transcript,
            },
        }
    }

    fn success(&self) -> ApiResponse {
        ApiResponse {
            ok: true,
            state: self.snapshot(),
            error: None,
            warning: None,
        }
    }

    fn success_with_warning(&self, warning: Option<String>) -> ApiResponse {
        ApiResponse {
            ok: true,
            state: self.snapshot(),
            error: None,
            warning,
        }
    }

    fn wrap(&self, result: Result<(), CalcError>) -> ApiResponse {
        match result {
            Ok(()) => self.success(),
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
                warning: None,
            },
        }
    }

    fn success_with_undo(&mut self, before: CalcState) -> ApiResponse {
        self.calculator.set_undo_state(before);
        self.success()
    }

    fn success_with_warning_and_undo(
        &mut self,
        before: CalcState,
        warning: Option<String>,
    ) -> ApiResponse {
        self.calculator.set_undo_state(before);
        self.success_with_warning(warning)
    }

    fn wrap_with_undo(&mut self, before: CalcState, result: Result<(), CalcError>) -> ApiResponse {
        match result {
            Ok(()) => self.success_with_undo(before),
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
                warning: None,
            },
        }
    }
}

fn to_api_value(value: &Value) -> ApiValue {
    match value {
        Value::Real(v) => ApiValue::Real { value: *v },
        Value::Complex(c) => ApiValue::Complex { re: c.re, im: c.im },
        Value::Matrix(m) => ApiValue::Matrix {
            rows: m.rows,
            cols: m.cols,
            data: m
                .data
                .iter()
                .map(|entry| ComplexInput {
                    re: entry.re,
                    im: entry.im,
                })
                .collect(),
        },
    }
}

fn to_api_state(state: &crate::CalcState) -> ApiState {
    let angle_mode = match state.angle_mode {
        AngleMode::Deg => ApiAngleMode::Deg,
        AngleMode::Rad => ApiAngleMode::Rad,
    };
    let display_mode = match state.display_mode {
        DisplayMode::Fix => ApiDisplayMode::Fix,
        DisplayMode::Sci => ApiDisplayMode::Sci,
        DisplayMode::Eng => ApiDisplayMode::Eng,
    };

    ApiState {
        stack: state.stack.iter().map(to_api_value).collect(),
        entry_buffer: state.entry_buffer.clone(),
        angle_mode,
        display_mode,
        precision: state.precision,
        memory: state
            .memory
            .iter()
            .map(|entry| entry.as_ref().map(to_api_value))
            .collect(),
    }
}

fn to_api_error(error: CalcError) -> ApiError {
    match error {
        CalcError::StackUnderflow { needed, available } => ApiError {
            code: "stack_underflow".to_string(),
            message: format!("stack underflow: needed {needed}, available {available}"),
        },
        CalcError::InvalidInput(message) => ApiError {
            code: "invalid_input".to_string(),
            message,
        },
        CalcError::DimensionMismatch { expected, actual } => ApiError {
            code: "dimension_mismatch".to_string(),
            message: format!("dimension mismatch: expected {expected}, actual {actual}"),
        },
        CalcError::TypeMismatch(message) => ApiError {
            code: "type_mismatch".to_string(),
            message,
        },
        CalcError::InvalidRegister(register) => ApiError {
            code: "invalid_register".to_string(),
            message: format!("invalid memory register index: {register}"),
        },
        CalcError::EmptyRegister(register) => ApiError {
            code: "empty_register".to_string(),
            message: format!("memory register is empty: {register}"),
        },
        CalcError::DomainError(message) => ApiError {
            code: "domain_error".to_string(),
            message,
        },
        CalcError::DivideByZero => ApiError {
            code: "divide_by_zero".to_string(),
            message: "divide by zero".to_string(),
        },
        CalcError::SingularMatrix(message) => ApiError {
            code: "singular_matrix".to_string(),
            message,
        },
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;

    use super::{ApiAngleMode, ApiDisplayMode, CalculatorApi, ComplexInput, MatrixInput};

    #[wasm_bindgen]
    pub struct WasmCalculator {
        inner: CalculatorApi,
    }

    #[wasm_bindgen]
    impl WasmCalculator {
        #[wasm_bindgen(constructor)]
        /// Constructs a new instance.
        pub fn new() -> Self {
            Self {
                inner: CalculatorApi::new(),
            }
        }

        /// Executes the `snapshot_json` operation.
        pub fn snapshot_json(&self) -> String {
            serde_json::to_string(&self.inner.snapshot())
                .expect("state serialization should succeed")
        }

        /// Executes the `entry_set` operation.
        pub fn entry_set(&mut self, value: &str) -> String {
            serde_json::to_string(&self.inner.entry_set(value))
                .expect("response serialization should succeed")
        }

        /// Executes the `enter` operation.
        pub fn enter(&mut self) -> String {
            serde_json::to_string(&self.inner.enter())
                .expect("response serialization should succeed")
        }

        /// Executes the `add` operation.
        pub fn add(&mut self) -> String {
            serde_json::to_string(&self.inner.add()).expect("response serialization should succeed")
        }

        /// Executes the `pow` operation.
        pub fn pow(&mut self) -> String {
            serde_json::to_string(&self.inner.pow()).expect("response serialization should succeed")
        }

        /// Executes the `percent` operation.
        pub fn percent(&mut self) -> String {
            serde_json::to_string(&self.inner.percent())
                .expect("response serialization should succeed")
        }

        /// Executes the `inv` operation.
        pub fn inv(&mut self) -> String {
            serde_json::to_string(&self.inner.inv()).expect("response serialization should succeed")
        }

        /// Executes the `square` operation.
        pub fn square(&mut self) -> String {
            serde_json::to_string(&self.inner.square())
                .expect("response serialization should succeed")
        }

        /// Executes the `root` operation.
        pub fn root(&mut self) -> String {
            serde_json::to_string(&self.inner.root())
                .expect("response serialization should succeed")
        }

        /// Executes the `drop` operation.
        pub fn drop(&mut self) -> String {
            serde_json::to_string(&self.inner.drop())
                .expect("response serialization should succeed")
        }

        /// Executes the `undo` operation.
        pub fn undo(&mut self) -> String {
            serde_json::to_string(&self.inner.undo())
                .expect("response serialization should succeed")
        }

        /// Executes the `dup` operation.
        pub fn dup(&mut self) -> String {
            serde_json::to_string(&self.inner.dup()).expect("response serialization should succeed")
        }

        /// Executes the `swap` operation.
        pub fn swap(&mut self) -> String {
            serde_json::to_string(&self.inner.swap())
                .expect("response serialization should succeed")
        }

        /// Executes the `rot` operation.
        pub fn rot(&mut self) -> String {
            serde_json::to_string(&self.inner.rot()).expect("response serialization should succeed")
        }

        /// Executes the `roll` operation.
        pub fn roll(&mut self, count: usize) -> String {
            serde_json::to_string(&self.inner.roll(count))
                .expect("response serialization should succeed")
        }

        /// Executes the `pick` operation.
        pub fn pick(&mut self, depth: usize) -> String {
            serde_json::to_string(&self.inner.pick(depth))
                .expect("response serialization should succeed")
        }

        /// Executes the `pick_from_stack_index` operation.
        pub fn pick_from_stack_index(&mut self) -> String {
            serde_json::to_string(&self.inner.pick_from_stack_index())
                .expect("response serialization should succeed")
        }

        /// Executes the `sub` operation.
        pub fn sub(&mut self) -> String {
            serde_json::to_string(&self.inner.sub()).expect("response serialization should succeed")
        }

        /// Executes the `mul` operation.
        pub fn mul(&mut self) -> String {
            serde_json::to_string(&self.inner.mul()).expect("response serialization should succeed")
        }

        /// Executes the `div` operation.
        pub fn div(&mut self) -> String {
            serde_json::to_string(&self.inner.div()).expect("response serialization should succeed")
        }

        /// Executes the `hadamard_mul` operation.
        pub fn hadamard_mul(&mut self) -> String {
            serde_json::to_string(&self.inner.hadamard_mul())
                .expect("response serialization should succeed")
        }

        /// Executes the `hadamard_div` operation.
        pub fn hadamard_div(&mut self) -> String {
            serde_json::to_string(&self.inner.hadamard_div())
                .expect("response serialization should succeed")
        }

        /// Executes the `sqrt` operation.
        pub fn sqrt(&mut self) -> String {
            serde_json::to_string(&self.inner.sqrt())
                .expect("response serialization should succeed")
        }

        /// Executes the `ln` operation.
        pub fn ln(&mut self) -> String {
            serde_json::to_string(&self.inner.ln()).expect("response serialization should succeed")
        }

        /// Executes the `sin` operation.
        pub fn sin(&mut self) -> String {
            serde_json::to_string(&self.inner.sin()).expect("response serialization should succeed")
        }

        /// Executes the `cos` operation.
        pub fn cos(&mut self) -> String {
            serde_json::to_string(&self.inner.cos()).expect("response serialization should succeed")
        }

        /// Executes the `tan` operation.
        pub fn tan(&mut self) -> String {
            serde_json::to_string(&self.inner.tan()).expect("response serialization should succeed")
        }

        /// Executes the `sec` operation.
        pub fn sec(&mut self) -> String {
            serde_json::to_string(&self.inner.sec()).expect("response serialization should succeed")
        }

        /// Executes the `csc` operation.
        pub fn csc(&mut self) -> String {
            serde_json::to_string(&self.inner.csc()).expect("response serialization should succeed")
        }

        /// Executes the `cot` operation.
        pub fn cot(&mut self) -> String {
            serde_json::to_string(&self.inner.cot()).expect("response serialization should succeed")
        }

        /// Executes the `asin` operation.
        pub fn asin(&mut self) -> String {
            serde_json::to_string(&self.inner.asin())
                .expect("response serialization should succeed")
        }

        /// Executes the `acos` operation.
        pub fn acos(&mut self) -> String {
            serde_json::to_string(&self.inner.acos())
                .expect("response serialization should succeed")
        }

        /// Executes the `asec` operation.
        pub fn asec(&mut self) -> String {
            serde_json::to_string(&self.inner.asec())
                .expect("response serialization should succeed")
        }

        /// Executes the `acsc` operation.
        pub fn acsc(&mut self) -> String {
            serde_json::to_string(&self.inner.acsc())
                .expect("response serialization should succeed")
        }

        /// Executes the `atan` operation.
        pub fn atan(&mut self) -> String {
            serde_json::to_string(&self.inner.atan())
                .expect("response serialization should succeed")
        }

        /// Executes the `acot` operation.
        pub fn acot(&mut self) -> String {
            serde_json::to_string(&self.inner.acot())
                .expect("response serialization should succeed")
        }

        /// Executes the `sinh` operation.
        pub fn sinh(&mut self) -> String {
            serde_json::to_string(&self.inner.sinh())
                .expect("response serialization should succeed")
        }

        /// Executes the `cosh` operation.
        pub fn cosh(&mut self) -> String {
            serde_json::to_string(&self.inner.cosh())
                .expect("response serialization should succeed")
        }

        /// Executes the `tanh` operation.
        pub fn tanh(&mut self) -> String {
            serde_json::to_string(&self.inner.tanh())
                .expect("response serialization should succeed")
        }

        /// Executes the `sech` operation.
        pub fn sech(&mut self) -> String {
            serde_json::to_string(&self.inner.sech())
                .expect("response serialization should succeed")
        }

        /// Executes the `csch` operation.
        pub fn csch(&mut self) -> String {
            serde_json::to_string(&self.inner.csch())
                .expect("response serialization should succeed")
        }

        /// Executes the `coth` operation.
        pub fn coth(&mut self) -> String {
            serde_json::to_string(&self.inner.coth())
                .expect("response serialization should succeed")
        }

        /// Executes the `asinh` operation.
        pub fn asinh(&mut self) -> String {
            serde_json::to_string(&self.inner.asinh())
                .expect("response serialization should succeed")
        }

        /// Executes the `acosh` operation.
        pub fn acosh(&mut self) -> String {
            serde_json::to_string(&self.inner.acosh())
                .expect("response serialization should succeed")
        }

        /// Executes the `asech` operation.
        pub fn asech(&mut self) -> String {
            serde_json::to_string(&self.inner.asech())
                .expect("response serialization should succeed")
        }

        /// Executes the `acsch` operation.
        pub fn acsch(&mut self) -> String {
            serde_json::to_string(&self.inner.acsch())
                .expect("response serialization should succeed")
        }

        /// Executes the `atanh` operation.
        pub fn atanh(&mut self) -> String {
            serde_json::to_string(&self.inner.atanh())
                .expect("response serialization should succeed")
        }

        /// Executes the `acoth` operation.
        pub fn acoth(&mut self) -> String {
            serde_json::to_string(&self.inner.acoth())
                .expect("response serialization should succeed")
        }

        /// Executes the `exp` operation.
        pub fn exp(&mut self) -> String {
            serde_json::to_string(&self.inner.exp()).expect("response serialization should succeed")
        }

        /// Executes the `exp10` operation.
        pub fn exp10(&mut self) -> String {
            serde_json::to_string(&self.inner.exp10())
                .expect("response serialization should succeed")
        }

        /// Executes the `exp2` operation.
        pub fn exp2(&mut self) -> String {
            serde_json::to_string(&self.inner.exp2())
                .expect("response serialization should succeed")
        }

        /// Executes the `log10` operation.
        pub fn log10(&mut self) -> String {
            serde_json::to_string(&self.inner.log10())
                .expect("response serialization should succeed")
        }

        /// Executes the `log2` operation.
        pub fn log2(&mut self) -> String {
            serde_json::to_string(&self.inner.log2())
                .expect("response serialization should succeed")
        }

        /// Executes the `log_y_x` operation.
        pub fn log_y_x(&mut self) -> String {
            serde_json::to_string(&self.inner.log_y_x())
                .expect("response serialization should succeed")
        }

        /// Executes the `gamma` operation.
        pub fn gamma(&mut self) -> String {
            serde_json::to_string(&self.inner.gamma())
                .expect("response serialization should succeed")
        }

        /// Executes the `erf` operation.
        pub fn erf(&mut self) -> String {
            serde_json::to_string(&self.inner.erf()).expect("response serialization should succeed")
        }

        /// Executes the `erfc` operation.
        pub fn erfc(&mut self) -> String {
            serde_json::to_string(&self.inner.erfc())
                .expect("response serialization should succeed")
        }

        /// Executes the `bessel` operation.
        pub fn bessel(&mut self) -> String {
            serde_json::to_string(&self.inner.bessel())
                .expect("response serialization should succeed")
        }

        /// Executes the `mbessel` operation.
        pub fn mbessel(&mut self) -> String {
            serde_json::to_string(&self.inner.mbessel())
                .expect("response serialization should succeed")
        }

        /// Executes the `sinc` operation.
        pub fn sinc(&mut self) -> String {
            serde_json::to_string(&self.inner.sinc())
                .expect("response serialization should succeed")
        }

        /// Executes the `neg` operation.
        pub fn neg(&mut self) -> String {
            serde_json::to_string(&self.inner.neg())
                .expect("response serialization should succeed")
        }

        /// Executes the `signum` operation.
        pub fn signum(&mut self) -> String {
            serde_json::to_string(&self.inner.signum())
                .expect("response serialization should succeed")
        }

        /// Executes the `abs` operation.
        pub fn abs(&mut self) -> String {
            serde_json::to_string(&self.inner.abs()).expect("response serialization should succeed")
        }

        /// Executes the `abs_sq` operation.
        pub fn abs_sq(&mut self) -> String {
            serde_json::to_string(&self.inner.abs_sq())
                .expect("response serialization should succeed")
        }

        /// Executes the `arg` operation.
        pub fn arg(&mut self) -> String {
            serde_json::to_string(&self.inner.arg()).expect("response serialization should succeed")
        }

        /// Executes the `conjugate` operation.
        pub fn conjugate(&mut self) -> String {
            serde_json::to_string(&self.inner.conjugate())
                .expect("response serialization should succeed")
        }

        /// Executes the `real_part` operation.
        pub fn real_part(&mut self) -> String {
            serde_json::to_string(&self.inner.real_part())
                .expect("response serialization should succeed")
        }

        /// Executes the `imag_part` operation.
        pub fn imag_part(&mut self) -> String {
            serde_json::to_string(&self.inner.imag_part())
                .expect("response serialization should succeed")
        }

        /// Executes the `cart` operation.
        pub fn cart(&mut self) -> String {
            serde_json::to_string(&self.inner.cart())
                .expect("response serialization should succeed")
        }

        /// Executes the `pol` operation.
        pub fn pol(&mut self) -> String {
            serde_json::to_string(&self.inner.pol()).expect("response serialization should succeed")
        }

        /// Executes the `npol` operation.
        pub fn npol(&mut self) -> String {
            serde_json::to_string(&self.inner.npol())
                .expect("response serialization should succeed")
        }

        /// Executes the `atan2` operation.
        pub fn atan2(&mut self) -> String {
            serde_json::to_string(&self.inner.atan2())
                .expect("response serialization should succeed")
        }

        /// Executes the `to_rad` operation.
        pub fn to_rad(&mut self) -> String {
            serde_json::to_string(&self.inner.to_rad())
                .expect("response serialization should succeed")
        }

        /// Executes the `to_deg` operation.
        pub fn to_deg(&mut self) -> String {
            serde_json::to_string(&self.inner.to_deg())
                .expect("response serialization should succeed")
        }

        /// Executes the `factorial` operation.
        pub fn factorial(&mut self) -> String {
            serde_json::to_string(&self.inner.factorial())
                .expect("response serialization should succeed")
        }

        /// Executes the `ncr` operation.
        pub fn ncr(&mut self) -> String {
            serde_json::to_string(&self.inner.ncr()).expect("response serialization should succeed")
        }

        /// Executes the `npr` operation.
        pub fn npr(&mut self) -> String {
            serde_json::to_string(&self.inner.npr()).expect("response serialization should succeed")
        }

        /// Executes the `modulo` operation.
        pub fn modulo(&mut self) -> String {
            serde_json::to_string(&self.inner.modulo())
                .expect("response serialization should succeed")
        }

        /// Executes the `rand_num` operation.
        pub fn rand_num(&mut self) -> String {
            serde_json::to_string(&self.inner.rand_num())
                .expect("response serialization should succeed")
        }

        /// Executes the `gcd` operation.
        pub fn gcd(&mut self) -> String {
            serde_json::to_string(&self.inner.gcd()).expect("response serialization should succeed")
        }

        /// Executes the `lcm` operation.
        pub fn lcm(&mut self) -> String {
            serde_json::to_string(&self.inner.lcm()).expect("response serialization should succeed")
        }

        /// Executes the `round_value` operation.
        pub fn round_value(&mut self) -> String {
            serde_json::to_string(&self.inner.round_value())
                .expect("response serialization should succeed")
        }

        /// Executes the `floor_value` operation.
        pub fn floor_value(&mut self) -> String {
            serde_json::to_string(&self.inner.floor_value())
                .expect("response serialization should succeed")
        }

        /// Executes the `ceil_value` operation.
        pub fn ceil_value(&mut self) -> String {
            serde_json::to_string(&self.inner.ceil_value())
                .expect("response serialization should succeed")
        }

        /// Executes the `dec_part` operation.
        pub fn dec_part(&mut self) -> String {
            serde_json::to_string(&self.inner.dec_part())
                .expect("response serialization should succeed")
        }

        /// Executes the `push_pi` operation.
        pub fn push_pi(&mut self) -> String {
            serde_json::to_string(&self.inner.push_pi())
                .expect("response serialization should succeed")
        }

        /// Executes the `push_e` operation.
        pub fn push_e(&mut self) -> String {
            serde_json::to_string(&self.inner.push_e())
                .expect("response serialization should succeed")
        }

        /// Executes the `set_angle_mode_deg` operation.
        pub fn set_angle_mode_deg(&mut self) -> String {
            serde_json::to_string(&self.inner.set_angle_mode(ApiAngleMode::Deg))
                .expect("response serialization should succeed")
        }

        /// Executes the `set_angle_mode_rad` operation.
        pub fn set_angle_mode_rad(&mut self) -> String {
            serde_json::to_string(&self.inner.set_angle_mode(ApiAngleMode::Rad))
                .expect("response serialization should succeed")
        }

        /// Executes the `set_display_mode_fix` operation.
        pub fn set_display_mode_fix(&mut self) -> String {
            serde_json::to_string(&self.inner.set_display_mode(ApiDisplayMode::Fix))
                .expect("response serialization should succeed")
        }

        /// Executes the `set_display_mode_sci` operation.
        pub fn set_display_mode_sci(&mut self) -> String {
            serde_json::to_string(&self.inner.set_display_mode(ApiDisplayMode::Sci))
                .expect("response serialization should succeed")
        }

        /// Executes the `set_display_mode_eng` operation.
        pub fn set_display_mode_eng(&mut self) -> String {
            serde_json::to_string(&self.inner.set_display_mode(ApiDisplayMode::Eng))
                .expect("response serialization should succeed")
        }

        /// Executes the `set_precision` operation.
        pub fn set_precision(&mut self, precision: u8) -> String {
            serde_json::to_string(&self.inner.set_precision(precision))
                .expect("response serialization should succeed")
        }

        /// Executes the `clear_entry` operation.
        pub fn clear_entry(&mut self) -> String {
            serde_json::to_string(&self.inner.clear_entry())
                .expect("response serialization should succeed")
        }

        /// Executes the `clear_all` operation.
        pub fn clear_all(&mut self) -> String {
            serde_json::to_string(&self.inner.clear_all())
                .expect("response serialization should succeed")
        }

        /// Executes the `push_real` operation.
        pub fn push_real(&mut self, value: f64) -> String {
            serde_json::to_string(&self.inner.push_real(value))
                .expect("response serialization should succeed")
        }

        /// Executes the `push_complex` operation.
        pub fn push_complex(&mut self, re: f64, im: f64) -> String {
            serde_json::to_string(&self.inner.push_complex(ComplexInput { re, im }))
                .expect("response serialization should succeed")
        }

        /// Executes the `push_matrix_json` operation.
        pub fn push_matrix_json(&mut self, matrix_json: &str) -> String {
            let parsed: Result<MatrixInput, _> = serde_json::from_str(matrix_json);
            match parsed {
                Ok(matrix) => serde_json::to_string(&self.inner.push_matrix(matrix))
                    .expect("response serialization should succeed"),
                Err(error) => serde_json::to_string(&super::ApiResponse {
                    ok: false,
                    state: self.inner.snapshot(),
                    error: Some(super::ApiError {
                        code: "invalid_input".to_string(),
                        message: format!("invalid matrix payload: {error}"),
                    }),
                    warning: None,
                })
                .expect("response serialization should succeed"),
            }
        }

        /// Executes the `determinant` operation.
        pub fn determinant(&mut self) -> String {
            serde_json::to_string(&self.inner.determinant())
                .expect("response serialization should succeed")
        }

        /// Executes the `inverse` operation.
        pub fn inverse(&mut self) -> String {
            serde_json::to_string(&self.inner.inverse())
                .expect("response serialization should succeed")
        }

        /// Executes the `transpose` operation.
        pub fn transpose(&mut self) -> String {
            serde_json::to_string(&self.inner.transpose())
                .expect("response serialization should succeed")
        }

        /// Executes the `solve_ax_b` operation.
        pub fn solve_ax_b(&mut self) -> String {
            serde_json::to_string(&self.inner.solve_ax_b())
                .expect("response serialization should succeed")
        }

        /// Executes the `solve_lstsq` operation.
        pub fn solve_lstsq(&mut self) -> String {
            serde_json::to_string(&self.inner.solve_lstsq())
                .expect("response serialization should succeed")
        }

        /// Executes the `dot` operation.
        pub fn dot(&mut self) -> String {
            serde_json::to_string(&self.inner.dot()).expect("response serialization should succeed")
        }

        /// Executes the `cross` operation.
        pub fn cross(&mut self) -> String {
            serde_json::to_string(&self.inner.cross())
                .expect("response serialization should succeed")
        }

        /// Executes the `trace` operation.
        pub fn trace(&mut self) -> String {
            serde_json::to_string(&self.inner.trace())
                .expect("response serialization should succeed")
        }

        /// Executes the `norm_p` operation.
        pub fn norm_p(&mut self) -> String {
            serde_json::to_string(&self.inner.norm_p())
                .expect("response serialization should succeed")
        }

        /// Executes the `diag` operation.
        pub fn diag(&mut self) -> String {
            serde_json::to_string(&self.inner.diag())
                .expect("response serialization should succeed")
        }

        /// Executes the `toep` operation.
        pub fn toep(&mut self) -> String {
            serde_json::to_string(&self.inner.toep())
                .expect("response serialization should succeed")
        }

        /// Executes the `mat_exp` operation.
        pub fn mat_exp(&mut self) -> String {
            serde_json::to_string(&self.inner.mat_exp())
                .expect("response serialization should succeed")
        }

        /// Executes the `hermitian` operation.
        pub fn hermitian(&mut self) -> String {
            serde_json::to_string(&self.inner.hermitian())
                .expect("response serialization should succeed")
        }

        /// Executes the `mat_pow` operation.
        pub fn mat_pow(&mut self) -> String {
            serde_json::to_string(&self.inner.mat_pow())
                .expect("response serialization should succeed")
        }

        /// Executes the `qr` operation.
        pub fn qr(&mut self) -> String {
            serde_json::to_string(&self.inner.qr()).expect("response serialization should succeed")
        }

        /// Executes the `lu` operation.
        pub fn lu(&mut self) -> String {
            serde_json::to_string(&self.inner.lu()).expect("response serialization should succeed")
        }

        /// Executes the `svd` operation.
        pub fn svd(&mut self) -> String {
            serde_json::to_string(&self.inner.svd()).expect("response serialization should succeed")
        }

        /// Executes the `evd` operation.
        pub fn evd(&mut self) -> String {
            serde_json::to_string(&self.inner.evd()).expect("response serialization should succeed")
        }

        /// Executes the `mean` operation.
        pub fn mean(&mut self) -> String {
            serde_json::to_string(&self.inner.mean())
                .expect("response serialization should succeed")
        }

        /// Executes the `mode` operation.
        pub fn mode(&mut self) -> String {
            serde_json::to_string(&self.inner.mode())
                .expect("response serialization should succeed")
        }

        /// Executes the `variance` operation.
        pub fn variance(&mut self) -> String {
            serde_json::to_string(&self.inner.variance())
                .expect("response serialization should succeed")
        }

        /// Executes the `std_dev_p` operation.
        pub fn std_dev_p(&mut self) -> String {
            serde_json::to_string(&self.inner.std_dev_p())
                .expect("response serialization should succeed")
        }

        /// Executes the `std_dev_s` operation.
        pub fn std_dev_s(&mut self) -> String {
            serde_json::to_string(&self.inner.std_dev_s())
                .expect("response serialization should succeed")
        }

        /// Executes the `median` operation.
        pub fn median(&mut self) -> String {
            serde_json::to_string(&self.inner.median())
                .expect("response serialization should succeed")
        }

        /// Executes the `quart` operation.
        pub fn quart(&mut self) -> String {
            serde_json::to_string(&self.inner.quart())
                .expect("response serialization should succeed")
        }

        /// Executes the `max_value` operation.
        pub fn max_value(&mut self) -> String {
            serde_json::to_string(&self.inner.max_value())
                .expect("response serialization should succeed")
        }

        /// Executes the `min_value` operation.
        pub fn min_value(&mut self) -> String {
            serde_json::to_string(&self.inner.min_value())
                .expect("response serialization should succeed")
        }

        /// Executes the `push_identity` operation.
        pub fn push_identity(&mut self, size: usize) -> String {
            serde_json::to_string(&self.inner.push_identity(size))
                .expect("response serialization should succeed")
        }

        /// Executes the `stack_vec` operation.
        pub fn stack_vec(&mut self) -> String {
            serde_json::to_string(&self.inner.stack_vec())
                .expect("response serialization should succeed")
        }

        /// Executes the `hstack` operation.
        pub fn hstack(&mut self) -> String {
            serde_json::to_string(&self.inner.hstack())
                .expect("response serialization should succeed")
        }

        /// Executes the `vstack` operation.
        pub fn vstack(&mut self) -> String {
            serde_json::to_string(&self.inner.vstack())
                .expect("response serialization should succeed")
        }

        /// Executes the `ravel` operation.
        pub fn ravel(&mut self) -> String {
            serde_json::to_string(&self.inner.ravel())
                .expect("response serialization should succeed")
        }

        /// Executes the `hravel` operation.
        pub fn hravel(&mut self) -> String {
            serde_json::to_string(&self.inner.hravel())
                .expect("response serialization should succeed")
        }

        /// Executes the `vravel` operation.
        pub fn vravel(&mut self) -> String {
            serde_json::to_string(&self.inner.vravel())
                .expect("response serialization should succeed")
        }

        /// Executes the `memory_store` operation.
        pub fn memory_store(&mut self, register: usize) -> String {
            serde_json::to_string(&self.inner.memory_store(register))
                .expect("response serialization should succeed")
        }

        /// Executes the `memory_recall` operation.
        pub fn memory_recall(&mut self, register: usize) -> String {
            serde_json::to_string(&self.inner.memory_recall(register))
                .expect("response serialization should succeed")
        }

        /// Executes the `memory_clear` operation.
        pub fn memory_clear(&mut self, register: usize) -> String {
            serde_json::to_string(&self.inner.memory_clear(register))
                .expect("response serialization should succeed")
        }

        /// Executes the `run_script` operation.
        pub fn run_script(&mut self, script_text: &str) -> String {
            serde_json::to_string(&self.inner.run_script(script_text))
                .expect("response serialization should succeed")
        }

        /// Executes the `run_script_line` operation.
        pub fn run_script_line(&mut self, script_line: &str) -> String {
            serde_json::to_string(&self.inner.run_script_line(script_line))
                .expect("response serialization should succeed")
        }
    }
}

#[cfg(test)]
#[path = "tests/api_tests.rs"]
mod tests;
