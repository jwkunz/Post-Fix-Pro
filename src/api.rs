use serde::{Deserialize, Serialize};

use crate::{AngleMode, CalcError, Calculator, Complex, DisplayMode, Matrix, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiValue {
    Real { value: f64 },
    Complex { re: f64, im: f64 },
    Matrix { rows: usize, cols: usize, data: Vec<f64> },
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixInput {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
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
        match Matrix::new(matrix.rows, matrix.cols, matrix.data) {
            Ok(value) => {
                self.calculator.push_value(Value::Matrix(value));
                self.success()
            }
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
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

    pub fn push_identity(&mut self, size: usize) -> ApiResponse {
        let result = self.calculator.push_identity(size);
        self.wrap(result)
    }

    fn success(&self) -> ApiResponse {
        ApiResponse {
            ok: true,
            state: self.snapshot(),
            error: None,
        }
    }

    fn wrap(&self, result: Result<(), CalcError>) -> ApiResponse {
        match result {
            Ok(()) => self.success(),
            Err(error) => ApiResponse {
                ok: false,
                state: self.snapshot(),
                error: Some(to_api_error(error)),
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
            data: m.data.clone(),
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
            serde_json::to_string(&self.inner.snapshot()).expect("state serialization should succeed")
        }

        pub fn entry_set(&mut self, value: &str) -> String {
            serde_json::to_string(&self.inner.entry_set(value))
                .expect("response serialization should succeed")
        }

        pub fn enter(&mut self) -> String {
            serde_json::to_string(&self.inner.enter()).expect("response serialization should succeed")
        }

        pub fn add(&mut self) -> String {
            serde_json::to_string(&self.inner.add()).expect("response serialization should succeed")
        }

        pub fn drop(&mut self) -> String {
            serde_json::to_string(&self.inner.drop()).expect("response serialization should succeed")
        }

        pub fn dup(&mut self) -> String {
            serde_json::to_string(&self.inner.dup()).expect("response serialization should succeed")
        }

        pub fn swap(&mut self) -> String {
            serde_json::to_string(&self.inner.swap()).expect("response serialization should succeed")
        }

        pub fn rot(&mut self) -> String {
            serde_json::to_string(&self.inner.rot()).expect("response serialization should succeed")
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
            serde_json::to_string(&self.inner.sqrt()).expect("response serialization should succeed")
        }

        pub fn ln(&mut self) -> String {
            serde_json::to_string(&self.inner.ln()).expect("response serialization should succeed")
        }

        pub fn sin(&mut self) -> String {
            serde_json::to_string(&self.inner.sin()).expect("response serialization should succeed")
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
            serde_json::to_string(&self.inner.clear_all()).expect("response serialization should succeed")
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
                })
                .expect("response serialization should succeed"),
            }
        }

        pub fn determinant(&mut self) -> String {
            serde_json::to_string(&self.inner.determinant())
                .expect("response serialization should succeed")
        }

        pub fn inverse(&mut self) -> String {
            serde_json::to_string(&self.inner.inverse()).expect("response serialization should succeed")
        }

        pub fn transpose(&mut self) -> String {
            serde_json::to_string(&self.inner.transpose())
                .expect("response serialization should succeed")
        }

        pub fn solve_ax_b(&mut self) -> String {
            serde_json::to_string(&self.inner.solve_ax_b())
                .expect("response serialization should succeed")
        }

        pub fn push_identity(&mut self, size: usize) -> String {
            serde_json::to_string(&self.inner.push_identity(size))
                .expect("response serialization should succeed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiAngleMode, ApiValue, CalculatorApi, ComplexInput, MatrixInput};

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
            data: vec![1.0, 2.0, 3.0, 4.0],
        };

        let response = api.push_matrix(matrix);

        assert!(response.ok);
        assert_eq!(
            response.state.stack,
            vec![ApiValue::Matrix {
                rows: 2,
                cols: 2,
                data: vec![1.0, 2.0, 3.0, 4.0]
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
                data: vec![1.0, 0.0, 0.0, 1.0]
            }]
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
}
