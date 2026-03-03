use serde::{Deserialize, Serialize};

use crate::{AngleMode, CalcError, Calculator, Complex, DisplayMode, Matrix, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiValue {
    Real {
        value: f64,
    },
    Complex {
        re: f64,
        im: f64,
    },
    Matrix {
        rows: usize,
        cols: usize,
        data: Vec<ComplexInput>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiAngleMode {
    Deg,
    Rad,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiDisplayMode {
    Fix,
    Sci,
    Eng,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiState {
    pub stack: Vec<ApiValue>,
    pub entry_buffer: String,
    pub angle_mode: ApiAngleMode,
    pub display_mode: ApiDisplayMode,
    pub precision: u8,
    pub memory: Vec<Option<ApiValue>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiResponse {
    pub ok: bool,
    pub state: ApiState,
    pub error: Option<ApiError>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixInput {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<ComplexInput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexInput {
    pub re: f64,
    pub im: f64,
}

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
    pub fn new() -> Self {
        Self {
            calculator: Calculator::new(),
        }
    }

    pub fn snapshot(&self) -> ApiState {
        to_api_state(self.calculator.state())
    }

    pub fn entry_set(&mut self, value: &str) -> ApiResponse {
        self.calculator.entry_set(value);
        self.success()
    }

    pub fn clear_entry(&mut self) -> ApiResponse {
        self.calculator.clear_entry();
        self.success()
    }

    pub fn clear_all(&mut self) -> ApiResponse {
        self.calculator.clear_all();
        self.success()
    }

    pub fn push_real(&mut self, value: f64) -> ApiResponse {
        self.calculator.push_value(Value::Real(value));
        self.success()
    }

    pub fn push_complex(&mut self, complex: ComplexInput) -> ApiResponse {
        self.calculator.push_value(Value::Complex(Complex {
            re: complex.re,
            im: complex.im,
        }));
        self.success()
    }

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
                self.calculator.push_value(Value::Matrix(value));
                self.success()
            }
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
                warning: None,
            },
        }
    }

    pub fn set_angle_mode(&mut self, mode: ApiAngleMode) -> ApiResponse {
        let mode = match mode {
            ApiAngleMode::Deg => AngleMode::Deg,
            ApiAngleMode::Rad => AngleMode::Rad,
        };
        self.calculator.set_angle_mode(mode);
        self.success()
    }

    pub fn enter(&mut self) -> ApiResponse {
        let result = self.calculator.enter();
        self.wrap(result)
    }

    pub fn drop(&mut self) -> ApiResponse {
        let result = self.calculator.drop().map(|_| ());
        self.wrap(result)
    }

    pub fn dup(&mut self) -> ApiResponse {
        let result = self.calculator.dup();
        self.wrap(result)
    }

    pub fn swap(&mut self) -> ApiResponse {
        let result = self.calculator.swap();
        self.wrap(result)
    }

    pub fn rot(&mut self) -> ApiResponse {
        let result = self.calculator.rot();
        self.wrap(result)
    }

    pub fn roll(&mut self, count: usize) -> ApiResponse {
        let result = self.calculator.roll(count);
        self.wrap(result)
    }

    pub fn pick(&mut self, depth: usize) -> ApiResponse {
        let result = self.calculator.pick(depth);
        self.wrap(result)
    }

    pub fn pow(&mut self) -> ApiResponse {
        let result = self.calculator.pow();
        self.wrap(result)
    }

    pub fn percent(&mut self) -> ApiResponse {
        let result = self.calculator.percent();
        self.wrap(result)
    }

    pub fn inv(&mut self) -> ApiResponse {
        let result = self.calculator.inv();
        self.wrap(result)
    }

    pub fn square(&mut self) -> ApiResponse {
        let result = self.calculator.square();
        self.wrap(result)
    }

    pub fn root(&mut self) -> ApiResponse {
        let result = self.calculator.root();
        self.wrap(result)
    }

    pub fn add(&mut self) -> ApiResponse {
        let result = self.calculator.add();
        self.wrap(result)
    }

    pub fn sub(&mut self) -> ApiResponse {
        let result = self.calculator.sub();
        self.wrap(result)
    }

    pub fn mul(&mut self) -> ApiResponse {
        let result = self.calculator.mul();
        self.wrap(result)
    }

    pub fn div(&mut self) -> ApiResponse {
        let result = self.calculator.div();
        self.wrap(result)
    }

    pub fn sqrt(&mut self) -> ApiResponse {
        let result = self.calculator.sqrt();
        self.wrap(result)
    }

    pub fn ln(&mut self) -> ApiResponse {
        let result = self.calculator.ln();
        self.wrap(result)
    }

    pub fn sin(&mut self) -> ApiResponse {
        let result = self.calculator.sin();
        self.wrap(result)
    }

    pub fn cos(&mut self) -> ApiResponse {
        let result = self.calculator.cos();
        self.wrap(result)
    }

    pub fn tan(&mut self) -> ApiResponse {
        let result = self.calculator.tan();
        self.wrap(result)
    }

    pub fn asin(&mut self) -> ApiResponse {
        let result = self.calculator.asin();
        self.wrap(result)
    }

    pub fn acos(&mut self) -> ApiResponse {
        let result = self.calculator.acos();
        self.wrap(result)
    }

    pub fn atan(&mut self) -> ApiResponse {
        let result = self.calculator.atan();
        self.wrap(result)
    }

    pub fn sinh(&mut self) -> ApiResponse {
        let result = self.calculator.sinh();
        self.wrap(result)
    }

    pub fn cosh(&mut self) -> ApiResponse {
        let result = self.calculator.cosh();
        self.wrap(result)
    }

    pub fn tanh(&mut self) -> ApiResponse {
        let result = self.calculator.tanh();
        self.wrap(result)
    }

    pub fn asinh(&mut self) -> ApiResponse {
        let result = self.calculator.asinh();
        self.wrap(result)
    }

    pub fn acosh(&mut self) -> ApiResponse {
        let result = self.calculator.acosh();
        self.wrap(result)
    }

    pub fn atanh(&mut self) -> ApiResponse {
        let result = self.calculator.atanh();
        self.wrap(result)
    }

    pub fn exp(&mut self) -> ApiResponse {
        let result = self.calculator.exp();
        self.wrap(result)
    }

    pub fn exp10(&mut self) -> ApiResponse {
        let result = self.calculator.exp10();
        self.wrap(result)
    }

    pub fn exp2(&mut self) -> ApiResponse {
        let result = self.calculator.exp2();
        self.wrap(result)
    }

    pub fn log10(&mut self) -> ApiResponse {
        let result = self.calculator.log10();
        self.wrap(result)
    }

    pub fn log2(&mut self) -> ApiResponse {
        let result = self.calculator.log2();
        self.wrap(result)
    }

    pub fn gamma(&mut self) -> ApiResponse {
        let result = self.calculator.gamma();
        self.wrap(result)
    }

    pub fn erf(&mut self) -> ApiResponse {
        let result = self.calculator.erf();
        self.wrap(result)
    }

    pub fn signum(&mut self) -> ApiResponse {
        let result = self.calculator.signum();
        self.wrap(result)
    }

    pub fn abs(&mut self) -> ApiResponse {
        let result = self.calculator.abs();
        self.wrap(result)
    }

    pub fn abs_sq(&mut self) -> ApiResponse {
        let result = self.calculator.abs_sq();
        self.wrap(result)
    }

    pub fn arg(&mut self) -> ApiResponse {
        let result = self.calculator.arg();
        self.wrap(result)
    }

    pub fn conjugate(&mut self) -> ApiResponse {
        let result = self.calculator.conjugate();
        self.wrap(result)
    }

    pub fn atan2(&mut self) -> ApiResponse {
        let result = self.calculator.atan2();
        self.wrap(result)
    }

    pub fn to_rad(&mut self) -> ApiResponse {
        let result = self.calculator.to_rad();
        self.wrap(result)
    }

    pub fn to_deg(&mut self) -> ApiResponse {
        let result = self.calculator.to_deg();
        self.wrap(result)
    }

    pub fn factorial(&mut self) -> ApiResponse {
        let result = self.calculator.factorial();
        self.wrap(result)
    }

    pub fn ncr(&mut self) -> ApiResponse {
        let result = self.calculator.ncr();
        self.wrap(result)
    }

    pub fn npr(&mut self) -> ApiResponse {
        let result = self.calculator.npr();
        self.wrap(result)
    }

    pub fn modulo(&mut self) -> ApiResponse {
        let result = self.calculator.modulo();
        self.wrap(result)
    }

    pub fn rand_num(&mut self) -> ApiResponse {
        let result = self.calculator.rand_num();
        self.wrap(result)
    }

    pub fn gcd(&mut self) -> ApiResponse {
        let result = self.calculator.gcd();
        self.wrap(result)
    }

    pub fn lcm(&mut self) -> ApiResponse {
        let result = self.calculator.lcm();
        self.wrap(result)
    }

    pub fn round_value(&mut self) -> ApiResponse {
        let result = self.calculator.round_value();
        self.wrap(result)
    }

    pub fn floor_value(&mut self) -> ApiResponse {
        let result = self.calculator.floor_value();
        self.wrap(result)
    }

    pub fn ceil_value(&mut self) -> ApiResponse {
        let result = self.calculator.ceil_value();
        self.wrap(result)
    }

    pub fn dec_part(&mut self) -> ApiResponse {
        let result = self.calculator.dec_part();
        self.wrap(result)
    }

    pub fn push_pi(&mut self) -> ApiResponse {
        self.calculator.push_pi();
        self.success()
    }

    pub fn push_e(&mut self) -> ApiResponse {
        self.calculator.push_e();
        self.success()
    }

    pub fn determinant(&mut self) -> ApiResponse {
        let result = self.calculator.determinant();
        self.wrap(result)
    }

    pub fn inverse(&mut self) -> ApiResponse {
        let result = self.calculator.inverse();
        self.wrap(result)
    }

    pub fn transpose(&mut self) -> ApiResponse {
        let result = self.calculator.transpose();
        self.wrap(result)
    }

    pub fn solve_ax_b(&mut self) -> ApiResponse {
        let result = self.calculator.solve_ax_b();
        self.wrap(result)
    }

    pub fn dot(&mut self) -> ApiResponse {
        let result = self.calculator.dot();
        self.wrap(result)
    }

    pub fn cross(&mut self) -> ApiResponse {
        let result = self.calculator.cross();
        self.wrap(result)
    }

    pub fn trace(&mut self) -> ApiResponse {
        let result = self.calculator.trace();
        self.wrap(result)
    }

    pub fn norm_p(&mut self) -> ApiResponse {
        let result = self.calculator.norm_p();
        self.wrap(result)
    }

    pub fn diag(&mut self) -> ApiResponse {
        let result = self.calculator.diag();
        self.wrap(result)
    }

    pub fn mat_exp(&mut self) -> ApiResponse {
        let result = self.calculator.mat_exp();
        self.wrap(result)
    }

    pub fn hermitian(&mut self) -> ApiResponse {
        let result = self.calculator.hermitian();
        self.wrap(result)
    }

    pub fn mat_pow(&mut self) -> ApiResponse {
        let result = self.calculator.mat_pow();
        self.wrap(result)
    }

    pub fn qr(&mut self) -> ApiResponse {
        let result = self.calculator.qr();
        self.wrap(result)
    }

    pub fn lu(&mut self) -> ApiResponse {
        let result = self.calculator.lu();
        self.wrap(result)
    }

    pub fn svd(&mut self) -> ApiResponse {
        let result = self.calculator.svd();
        self.wrap(result)
    }

    pub fn evd(&mut self) -> ApiResponse {
        match self.calculator.evd() {
            Ok(warning) => self.success_with_warning(warning),
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
                warning: None,
            },
        }
    }

    pub fn mean(&mut self) -> ApiResponse {
        let result = self.calculator.mean();
        self.wrap(result)
    }

    pub fn mode(&mut self) -> ApiResponse {
        let result = self.calculator.mode();
        self.wrap(result)
    }

    pub fn variance(&mut self) -> ApiResponse {
        let result = self.calculator.variance();
        self.wrap(result)
    }

    pub fn std_dev(&mut self) -> ApiResponse {
        let result = self.calculator.std_dev();
        self.wrap(result)
    }

    pub fn max_value(&mut self) -> ApiResponse {
        let result = self.calculator.max_value();
        self.wrap(result)
    }

    pub fn min_value(&mut self) -> ApiResponse {
        let result = self.calculator.min_value();
        self.wrap(result)
    }

    pub fn push_identity(&mut self, size: usize) -> ApiResponse {
        let result = self.calculator.push_identity(size);
        self.wrap(result)
    }

    pub fn memory_store(&mut self, register: usize) -> ApiResponse {
        let result = self.calculator.memory_store(register);
        self.wrap(result)
    }

    pub fn memory_recall(&mut self, register: usize) -> ApiResponse {
        let result = self.calculator.memory_recall(register);
        self.wrap(result)
    }

    pub fn memory_clear(&mut self, register: usize) -> ApiResponse {
        let result = self.calculator.memory_clear(register);
        self.wrap(result)
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

    use super::{ApiAngleMode, CalculatorApi, ComplexInput, MatrixInput};

    #[wasm_bindgen]
    pub struct WasmCalculator {
        inner: CalculatorApi,
    }

    #[wasm_bindgen]
    impl WasmCalculator {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                inner: CalculatorApi::new(),
            }
        }

        pub fn snapshot_json(&self) -> String {
            serde_json::to_string(&self.inner.snapshot())
                .expect("state serialization should succeed")
        }

        pub fn entry_set(&mut self, value: &str) -> String {
            serde_json::to_string(&self.inner.entry_set(value))
                .expect("response serialization should succeed")
        }

        pub fn enter(&mut self) -> String {
            serde_json::to_string(&self.inner.enter())
                .expect("response serialization should succeed")
        }

        pub fn add(&mut self) -> String {
            serde_json::to_string(&self.inner.add()).expect("response serialization should succeed")
        }

        pub fn pow(&mut self) -> String {
            serde_json::to_string(&self.inner.pow()).expect("response serialization should succeed")
        }

        pub fn percent(&mut self) -> String {
            serde_json::to_string(&self.inner.percent())
                .expect("response serialization should succeed")
        }

        pub fn inv(&mut self) -> String {
            serde_json::to_string(&self.inner.inv()).expect("response serialization should succeed")
        }

        pub fn square(&mut self) -> String {
            serde_json::to_string(&self.inner.square())
                .expect("response serialization should succeed")
        }

        pub fn root(&mut self) -> String {
            serde_json::to_string(&self.inner.root())
                .expect("response serialization should succeed")
        }

        pub fn drop(&mut self) -> String {
            serde_json::to_string(&self.inner.drop())
                .expect("response serialization should succeed")
        }

        pub fn dup(&mut self) -> String {
            serde_json::to_string(&self.inner.dup()).expect("response serialization should succeed")
        }

        pub fn swap(&mut self) -> String {
            serde_json::to_string(&self.inner.swap())
                .expect("response serialization should succeed")
        }

        pub fn rot(&mut self) -> String {
            serde_json::to_string(&self.inner.rot()).expect("response serialization should succeed")
        }

        pub fn roll(&mut self, count: usize) -> String {
            serde_json::to_string(&self.inner.roll(count))
                .expect("response serialization should succeed")
        }

        pub fn pick(&mut self, depth: usize) -> String {
            serde_json::to_string(&self.inner.pick(depth))
                .expect("response serialization should succeed")
        }

        pub fn sub(&mut self) -> String {
            serde_json::to_string(&self.inner.sub()).expect("response serialization should succeed")
        }

        pub fn mul(&mut self) -> String {
            serde_json::to_string(&self.inner.mul()).expect("response serialization should succeed")
        }

        pub fn div(&mut self) -> String {
            serde_json::to_string(&self.inner.div()).expect("response serialization should succeed")
        }

        pub fn sqrt(&mut self) -> String {
            serde_json::to_string(&self.inner.sqrt())
                .expect("response serialization should succeed")
        }

        pub fn ln(&mut self) -> String {
            serde_json::to_string(&self.inner.ln()).expect("response serialization should succeed")
        }

        pub fn sin(&mut self) -> String {
            serde_json::to_string(&self.inner.sin()).expect("response serialization should succeed")
        }

        pub fn cos(&mut self) -> String {
            serde_json::to_string(&self.inner.cos()).expect("response serialization should succeed")
        }

        pub fn tan(&mut self) -> String {
            serde_json::to_string(&self.inner.tan()).expect("response serialization should succeed")
        }

        pub fn asin(&mut self) -> String {
            serde_json::to_string(&self.inner.asin())
                .expect("response serialization should succeed")
        }

        pub fn acos(&mut self) -> String {
            serde_json::to_string(&self.inner.acos())
                .expect("response serialization should succeed")
        }

        pub fn atan(&mut self) -> String {
            serde_json::to_string(&self.inner.atan())
                .expect("response serialization should succeed")
        }

        pub fn sinh(&mut self) -> String {
            serde_json::to_string(&self.inner.sinh())
                .expect("response serialization should succeed")
        }

        pub fn cosh(&mut self) -> String {
            serde_json::to_string(&self.inner.cosh())
                .expect("response serialization should succeed")
        }

        pub fn tanh(&mut self) -> String {
            serde_json::to_string(&self.inner.tanh())
                .expect("response serialization should succeed")
        }

        pub fn asinh(&mut self) -> String {
            serde_json::to_string(&self.inner.asinh())
                .expect("response serialization should succeed")
        }

        pub fn acosh(&mut self) -> String {
            serde_json::to_string(&self.inner.acosh())
                .expect("response serialization should succeed")
        }

        pub fn atanh(&mut self) -> String {
            serde_json::to_string(&self.inner.atanh())
                .expect("response serialization should succeed")
        }

        pub fn exp(&mut self) -> String {
            serde_json::to_string(&self.inner.exp()).expect("response serialization should succeed")
        }

        pub fn exp10(&mut self) -> String {
            serde_json::to_string(&self.inner.exp10())
                .expect("response serialization should succeed")
        }

        pub fn exp2(&mut self) -> String {
            serde_json::to_string(&self.inner.exp2())
                .expect("response serialization should succeed")
        }

        pub fn log10(&mut self) -> String {
            serde_json::to_string(&self.inner.log10())
                .expect("response serialization should succeed")
        }

        pub fn log2(&mut self) -> String {
            serde_json::to_string(&self.inner.log2())
                .expect("response serialization should succeed")
        }

        pub fn gamma(&mut self) -> String {
            serde_json::to_string(&self.inner.gamma())
                .expect("response serialization should succeed")
        }

        pub fn erf(&mut self) -> String {
            serde_json::to_string(&self.inner.erf()).expect("response serialization should succeed")
        }

        pub fn signum(&mut self) -> String {
            serde_json::to_string(&self.inner.signum())
                .expect("response serialization should succeed")
        }

        pub fn abs(&mut self) -> String {
            serde_json::to_string(&self.inner.abs()).expect("response serialization should succeed")
        }

        pub fn abs_sq(&mut self) -> String {
            serde_json::to_string(&self.inner.abs_sq())
                .expect("response serialization should succeed")
        }

        pub fn arg(&mut self) -> String {
            serde_json::to_string(&self.inner.arg()).expect("response serialization should succeed")
        }

        pub fn conjugate(&mut self) -> String {
            serde_json::to_string(&self.inner.conjugate())
                .expect("response serialization should succeed")
        }

        pub fn atan2(&mut self) -> String {
            serde_json::to_string(&self.inner.atan2())
                .expect("response serialization should succeed")
        }

        pub fn to_rad(&mut self) -> String {
            serde_json::to_string(&self.inner.to_rad())
                .expect("response serialization should succeed")
        }

        pub fn to_deg(&mut self) -> String {
            serde_json::to_string(&self.inner.to_deg())
                .expect("response serialization should succeed")
        }

        pub fn factorial(&mut self) -> String {
            serde_json::to_string(&self.inner.factorial())
                .expect("response serialization should succeed")
        }

        pub fn ncr(&mut self) -> String {
            serde_json::to_string(&self.inner.ncr()).expect("response serialization should succeed")
        }

        pub fn npr(&mut self) -> String {
            serde_json::to_string(&self.inner.npr()).expect("response serialization should succeed")
        }

        pub fn modulo(&mut self) -> String {
            serde_json::to_string(&self.inner.modulo())
                .expect("response serialization should succeed")
        }

        pub fn rand_num(&mut self) -> String {
            serde_json::to_string(&self.inner.rand_num())
                .expect("response serialization should succeed")
        }

        pub fn gcd(&mut self) -> String {
            serde_json::to_string(&self.inner.gcd()).expect("response serialization should succeed")
        }

        pub fn lcm(&mut self) -> String {
            serde_json::to_string(&self.inner.lcm()).expect("response serialization should succeed")
        }

        pub fn round_value(&mut self) -> String {
            serde_json::to_string(&self.inner.round_value())
                .expect("response serialization should succeed")
        }

        pub fn floor_value(&mut self) -> String {
            serde_json::to_string(&self.inner.floor_value())
                .expect("response serialization should succeed")
        }

        pub fn ceil_value(&mut self) -> String {
            serde_json::to_string(&self.inner.ceil_value())
                .expect("response serialization should succeed")
        }

        pub fn dec_part(&mut self) -> String {
            serde_json::to_string(&self.inner.dec_part())
                .expect("response serialization should succeed")
        }

        pub fn push_pi(&mut self) -> String {
            serde_json::to_string(&self.inner.push_pi())
                .expect("response serialization should succeed")
        }

        pub fn push_e(&mut self) -> String {
            serde_json::to_string(&self.inner.push_e())
                .expect("response serialization should succeed")
        }

        pub fn set_angle_mode_deg(&mut self) -> String {
            serde_json::to_string(&self.inner.set_angle_mode(ApiAngleMode::Deg))
                .expect("response serialization should succeed")
        }

        pub fn set_angle_mode_rad(&mut self) -> String {
            serde_json::to_string(&self.inner.set_angle_mode(ApiAngleMode::Rad))
                .expect("response serialization should succeed")
        }

        pub fn clear_entry(&mut self) -> String {
            serde_json::to_string(&self.inner.clear_entry())
                .expect("response serialization should succeed")
        }

        pub fn clear_all(&mut self) -> String {
            serde_json::to_string(&self.inner.clear_all())
                .expect("response serialization should succeed")
        }

        pub fn push_real(&mut self, value: f64) -> String {
            serde_json::to_string(&self.inner.push_real(value))
                .expect("response serialization should succeed")
        }

        pub fn push_complex(&mut self, re: f64, im: f64) -> String {
            serde_json::to_string(&self.inner.push_complex(ComplexInput { re, im }))
                .expect("response serialization should succeed")
        }

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

        pub fn determinant(&mut self) -> String {
            serde_json::to_string(&self.inner.determinant())
                .expect("response serialization should succeed")
        }

        pub fn inverse(&mut self) -> String {
            serde_json::to_string(&self.inner.inverse())
                .expect("response serialization should succeed")
        }

        pub fn transpose(&mut self) -> String {
            serde_json::to_string(&self.inner.transpose())
                .expect("response serialization should succeed")
        }

        pub fn solve_ax_b(&mut self) -> String {
            serde_json::to_string(&self.inner.solve_ax_b())
                .expect("response serialization should succeed")
        }

        pub fn dot(&mut self) -> String {
            serde_json::to_string(&self.inner.dot()).expect("response serialization should succeed")
        }

        pub fn cross(&mut self) -> String {
            serde_json::to_string(&self.inner.cross())
                .expect("response serialization should succeed")
        }

        pub fn trace(&mut self) -> String {
            serde_json::to_string(&self.inner.trace())
                .expect("response serialization should succeed")
        }

        pub fn norm_p(&mut self) -> String {
            serde_json::to_string(&self.inner.norm_p())
                .expect("response serialization should succeed")
        }

        pub fn diag(&mut self) -> String {
            serde_json::to_string(&self.inner.diag())
                .expect("response serialization should succeed")
        }

        pub fn mat_exp(&mut self) -> String {
            serde_json::to_string(&self.inner.mat_exp())
                .expect("response serialization should succeed")
        }

        pub fn hermitian(&mut self) -> String {
            serde_json::to_string(&self.inner.hermitian())
                .expect("response serialization should succeed")
        }

        pub fn mat_pow(&mut self) -> String {
            serde_json::to_string(&self.inner.mat_pow())
                .expect("response serialization should succeed")
        }

        pub fn qr(&mut self) -> String {
            serde_json::to_string(&self.inner.qr()).expect("response serialization should succeed")
        }

        pub fn lu(&mut self) -> String {
            serde_json::to_string(&self.inner.lu()).expect("response serialization should succeed")
        }

        pub fn svd(&mut self) -> String {
            serde_json::to_string(&self.inner.svd()).expect("response serialization should succeed")
        }

        pub fn evd(&mut self) -> String {
            serde_json::to_string(&self.inner.evd()).expect("response serialization should succeed")
        }

        pub fn mean(&mut self) -> String {
            serde_json::to_string(&self.inner.mean())
                .expect("response serialization should succeed")
        }

        pub fn mode(&mut self) -> String {
            serde_json::to_string(&self.inner.mode())
                .expect("response serialization should succeed")
        }

        pub fn variance(&mut self) -> String {
            serde_json::to_string(&self.inner.variance())
                .expect("response serialization should succeed")
        }

        pub fn std_dev(&mut self) -> String {
            serde_json::to_string(&self.inner.std_dev())
                .expect("response serialization should succeed")
        }

        pub fn max_value(&mut self) -> String {
            serde_json::to_string(&self.inner.max_value())
                .expect("response serialization should succeed")
        }

        pub fn min_value(&mut self) -> String {
            serde_json::to_string(&self.inner.min_value())
                .expect("response serialization should succeed")
        }

        pub fn push_identity(&mut self, size: usize) -> String {
            serde_json::to_string(&self.inner.push_identity(size))
                .expect("response serialization should succeed")
        }

        pub fn memory_store(&mut self, register: usize) -> String {
            serde_json::to_string(&self.inner.memory_store(register))
                .expect("response serialization should succeed")
        }

        pub fn memory_recall(&mut self, register: usize) -> String {
            serde_json::to_string(&self.inner.memory_recall(register))
                .expect("response serialization should succeed")
        }

        pub fn memory_clear(&mut self, register: usize) -> String {
            serde_json::to_string(&self.inner.memory_clear(register))
                .expect("response serialization should succeed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiAngleMode, ApiValue, CalculatorApi, ComplexInput, MatrixInput};

    fn c(re: f64, im: f64) -> ComplexInput {
        ComplexInput { re, im }
    }

    #[test]
    fn successful_operation_returns_ok_with_updated_state() {
        let mut api = CalculatorApi::new();
        api.push_real(2.0);
        api.push_real(3.0);

        let response = api.add();

        assert!(response.ok);
        assert_eq!(response.error, None);
        assert_eq!(response.state.stack, vec![ApiValue::Real { value: 5.0 }]);
    }

    #[test]
    fn failing_operation_returns_error_and_preserved_state() {
        let mut api = CalculatorApi::new();
        api.push_real(4.0);
        api.push_real(0.0);

        let response = api.div();

        assert!(!response.ok);
        let error = response.error.expect("error expected");
        assert_eq!(error.code, "divide_by_zero");
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Real { value: 4.0 }, ApiValue::Real { value: 0.0 }]
        );
    }

    #[test]
    fn snapshot_contains_mode_and_entry_state() {
        let mut api = CalculatorApi::new();
        api.entry_set("90");
        api.set_angle_mode(ApiAngleMode::Deg);

        let snapshot = api.snapshot();

        assert_eq!(snapshot.entry_buffer, "90");
        assert_eq!(snapshot.angle_mode, ApiAngleMode::Deg);
    }

    #[test]
    fn push_matrix_adds_matrix_to_stack() {
        let mut api = CalculatorApi::new();
        let matrix = MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        };

        let response = api.push_matrix(matrix);

        assert!(response.ok);
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Matrix {
                rows: 2,
                cols: 2,
                data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)]
            }]
        );
    }

    #[test]
    fn push_complex_adds_complex_to_stack() {
        let mut api = CalculatorApi::new();

        let response = api.push_complex(ComplexInput { re: 1.5, im: -2.0 });

        assert!(response.ok);
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Complex { re: 1.5, im: -2.0 }]
        );
    }

    #[test]
    fn push_identity_adds_identity_matrix() {
        let mut api = CalculatorApi::new();

        let response = api.push_identity(2);

        assert!(response.ok);
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Matrix {
                rows: 2,
                cols: 2,
                data: vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(1.0, 0.0)]
            }]
        );
    }

    #[test]
    fn dot_trace_and_norm_p_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)],
        });
        api.push_matrix(MatrixInput {
            rows: 3,
            cols: 1,
            data: vec![c(4.0, 0.0), c(5.0, 0.0), c(6.0, 0.0)],
        });

        let dot_response = api.dot();
        assert!(dot_response.ok);
        assert_eq!(
            dot_response.state.stack,
            vec![ApiValue::Complex { re: 32.0, im: 0.0 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });

        let trace_response = api.trace();
        assert!(trace_response.ok);
        assert_eq!(
            trace_response.state.stack,
            vec![ApiValue::Complex { re: 5.0, im: 0.0 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 2,
            data: vec![c(3.0, 0.0), c(4.0, 0.0)],
        });
        api.push_real(2.0);

        let norm_response = api.norm_p();
        assert!(norm_response.ok);
        assert_eq!(
            norm_response.state.stack,
            vec![ApiValue::Real { value: 5.0 }]
        );
    }

    #[test]
    fn diag_and_mat_exp_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 3,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)],
        });

        let diag_response = api.diag();
        assert!(diag_response.ok);
        assert_eq!(
            diag_response.state.stack,
            vec![ApiValue::Matrix {
                rows: 3,
                cols: 3,
                data: vec![
                    c(1.0, 0.0),
                    c(0.0, 0.0),
                    c(0.0, 0.0),
                    c(0.0, 0.0),
                    c(2.0, 0.0),
                    c(0.0, 0.0),
                    c(0.0, 0.0),
                    c(0.0, 0.0),
                    c(3.0, 0.0),
                ]
            }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(2.0, 0.0)],
        });

        let exp_response = api.mat_exp();
        assert!(exp_response.ok);
        match exp_response.state.stack.as_slice() {
            [ApiValue::Matrix { rows, cols, data }] => {
                assert_eq!((*rows, *cols), (2, 2));
                assert!((data[0].re - std::f64::consts::E).abs() < 1e-10);
                assert!(data[1].re.abs() < 1e-10);
                assert!(data[2].re.abs() < 1e-10);
                assert!((data[3].re - std::f64::consts::E.powi(2)).abs() < 1e-10);
            }
            other => panic!("expected matrix response, got {other:?}"),
        }
    }

    #[test]
    fn hermitian_and_mat_pow_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 2.0), c(3.0, -1.0), c(-4.0, 0.5), c(2.0, 0.0)],
        });

        let herm_response = api.hermitian();
        assert!(herm_response.ok);
        assert_eq!(herm_response.state.stack.len(), 1);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(2.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(3.0, 0.0)],
        });
        api.push_real(3.0);

        let pow_response = api.mat_pow();
        assert!(pow_response.ok);
        match pow_response.state.stack.as_slice() {
            [ApiValue::Matrix { rows, cols, data }] => {
                assert_eq!((*rows, *cols), (2, 2));
                assert!((data[0].re - 8.0).abs() < 1e-12);
                assert!((data[3].re - 27.0).abs() < 1e-12);
            }
            other => panic!("expected matrix mat_pow output, got {other:?}"),
        }
    }

    #[test]
    fn qr_and_lu_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
        });

        let qr_response = api.qr();
        assert!(qr_response.ok);
        assert_eq!(qr_response.state.stack.len(), 2);

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(4.0, 0.0), c(3.0, 0.0), c(6.0, 0.0), c(3.0, 0.0)],
        });

        let lu_response = api.lu();
        assert!(lu_response.ok);
        assert_eq!(lu_response.state.stack.len(), 3);
    }

    #[test]
    fn svd_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(3.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(3.0, 0.0)],
        });

        let response = api.svd();
        assert!(response.ok);
        assert_eq!(response.state.stack.len(), 3);
        match response.state.stack.as_slice() {
            [
                ApiValue::Matrix {
                    rows: ur, cols: uc, ..
                },
                ApiValue::Matrix {
                    rows: sr, cols: sc, ..
                },
                ApiValue::Matrix {
                    rows: vr, cols: vc, ..
                },
            ] => {
                assert_eq!((*ur, *uc), (2, 2));
                assert_eq!((*sr, *sc), (2, 2));
                assert_eq!((*vr, *vc), (2, 2));
            }
            other => panic!("expected three matrices from svd, got {other:?}"),
        }
    }

    #[test]
    fn evd_work_via_api_with_warning() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 2,
            cols: 2,
            data: vec![c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0), c(1.0, 0.0)],
        });

        let response = api.evd();
        assert!(response.ok);
        assert_eq!(response.state.stack.len(), 2);
        assert!(response.warning.is_some());
    }

    #[test]
    fn vector_statistics_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 5,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(2.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
            ],
        });

        let mean_response = api.mean();
        assert!(mean_response.ok);
        assert_eq!(
            mean_response.state.stack,
            vec![ApiValue::Real { value: 2.8 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 5,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(2.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
            ],
        });

        let mode_response = api.mode();
        assert!(mode_response.ok);
        assert_eq!(
            mode_response.state.stack,
            vec![ApiValue::Real { value: 2.0 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 2,
            data: vec![c(3.0, 0.0), c(4.0, 0.0)],
        });

        let variance_response = api.variance();
        assert!(variance_response.ok);
        assert_eq!(
            variance_response.state.stack,
            vec![ApiValue::Real { value: 0.25 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 2,
            data: vec![c(3.0, 0.0), c(4.0, 0.0)],
        });

        let std_response = api.std_dev();
        assert!(std_response.ok);
        assert_eq!(
            std_response.state.stack,
            vec![ApiValue::Real { value: 0.5 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 5,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(2.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
            ],
        });

        let max_response = api.max_value();
        assert!(max_response.ok);
        assert_eq!(
            max_response.state.stack,
            vec![ApiValue::Real { value: 5.0 }]
        );

        api.clear_all();
        api.push_matrix(MatrixInput {
            rows: 1,
            cols: 5,
            data: vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(2.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
            ],
        });

        let min_response = api.min_value();
        assert!(min_response.ok);
        assert_eq!(
            min_response.state.stack,
            vec![ApiValue::Real { value: 1.0 }]
        );
    }

    #[test]
    fn stack_utility_operations_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_real(1.0);
        api.push_real(2.0);
        api.push_real(3.0);

        let swap_response = api.swap();
        assert!(swap_response.ok);
        assert_eq!(
            swap_response.state.stack,
            vec![
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 2.0 }
            ]
        );

        let dup_response = api.dup();
        assert!(dup_response.ok);
        assert_eq!(
            dup_response.state.stack,
            vec![
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 2.0 },
                ApiValue::Real { value: 2.0 }
            ]
        );

        let drop_response = api.drop();
        assert!(drop_response.ok);
        assert_eq!(
            drop_response.state.stack,
            vec![
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 2.0 }
            ]
        );
    }

    #[test]
    fn roll_and_pick_work_via_api() {
        let mut api = CalculatorApi::new();
        api.push_real(1.0);
        api.push_real(2.0);
        api.push_real(3.0);
        api.push_real(4.0);

        let roll_response = api.roll(4);
        assert!(roll_response.ok);
        assert_eq!(
            roll_response.state.stack,
            vec![
                ApiValue::Real { value: 2.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 4.0 },
                ApiValue::Real { value: 1.0 }
            ]
        );

        let pick_response = api.pick(2);
        assert!(pick_response.ok);
        assert_eq!(
            pick_response.state.stack,
            vec![
                ApiValue::Real { value: 2.0 },
                ApiValue::Real { value: 3.0 },
                ApiValue::Real { value: 4.0 },
                ApiValue::Real { value: 1.0 },
                ApiValue::Real { value: 4.0 }
            ]
        );
    }
}
