use nalgebra::linalg::Schur;
use nalgebra::{DMatrix, DVector, Vector3};
use num_complex::Complex64;

pub mod api;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Real(f64),
    Complex(Complex),
    Matrix(Matrix),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Complex>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<Complex>) -> Result<Self, CalcError> {
        if rows == 0 || cols == 0 {
            return Err(CalcError::InvalidInput(
                "matrix dimensions must be non-zero".to_string(),
            ));
        }

        if rows * cols != data.len() {
            return Err(CalcError::DimensionMismatch {
                expected: rows * cols,
                actual: data.len(),
            });
        }

        Ok(Self { rows, cols, data })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleMode {
    Deg,
    Rad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Fix,
    Sci,
    Eng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComplexTransformMode {
    Cartesian,
    Polar,
    NormalizedPolar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    StackUnderflow { needed: usize, available: usize },
    InvalidInput(String),
    DimensionMismatch { expected: usize, actual: usize },
    TypeMismatch(String),
    InvalidRegister(usize),
    EmptyRegister(usize),
    DomainError(String),
    DivideByZero,
    SingularMatrix(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcState {
    pub stack: Vec<Value>,
    pub entry_buffer: String,
    pub angle_mode: AngleMode,
    pub display_mode: DisplayMode,
    pub precision: u8,
    pub memory: Vec<Option<Value>>,
    rng_state: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Calculator {
    state: CalcState,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            state: CalcState {
                stack: Vec::new(),
                entry_buffer: String::new(),
                angle_mode: AngleMode::Rad,
                display_mode: DisplayMode::Fix,
                precision: 6,
                memory: vec![None; 26],
                rng_state: 0x9E37_79B9_7F4A_7C15,
            },
        }
    }

    pub fn state(&self) -> &CalcState {
        &self.state
    }

    pub fn set_angle_mode(&mut self, mode: AngleMode) {
        self.state.angle_mode = mode;
    }

    pub fn push_pi(&mut self) {
        self.state.stack.push(Value::Real(std::f64::consts::PI));
    }

    pub fn push_e(&mut self) {
        self.state.stack.push(Value::Real(std::f64::consts::E));
    }

    pub fn entry_set(&mut self, value: &str) {
        self.state.entry_buffer = value.to_string();
    }

    pub fn clear_entry(&mut self) {
        self.state.entry_buffer.clear();
    }

    pub fn clear_all(&mut self) {
        self.state.stack.clear();
        self.state.entry_buffer.clear();
    }

    pub fn push_value(&mut self, value: Value) {
        self.state.stack.push(value);
    }

    pub fn enter(&mut self) -> Result<(), CalcError> {
        if self.state.entry_buffer.trim().is_empty() {
            return Err(CalcError::InvalidInput("entry buffer is empty".to_string()));
        }

        let value = self.state.entry_buffer.parse::<f64>().map_err(|_| {
            CalcError::InvalidInput("entry buffer is not a valid number".to_string())
        })?;

        self.state.stack.push(Value::Real(value));
        self.state.entry_buffer.clear();
        Ok(())
    }

    pub fn drop(&mut self) -> Result<Value, CalcError> {
        self.require_stack_len(1)?;
        Ok(self.state.stack.pop().expect("prechecked stack length"))
    }

    pub fn dup(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let top = self
            .state
            .stack
            .last()
            .expect("prechecked stack length")
            .clone();
        self.state.stack.push(top);
        Ok(())
    }

    pub fn swap(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(2)?;
        let len = self.state.stack.len();
        self.state.stack.swap(len - 1, len - 2);
        Ok(())
    }

    pub fn rot(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(3)?;
        let len = self.state.stack.len();
        self.state.stack[len - 3..].rotate_left(1);
        Ok(())
    }

    pub fn roll(&mut self, count: usize) -> Result<(), CalcError> {
        if count < 2 {
            return Err(CalcError::InvalidInput(
                "roll count must be at least 2".to_string(),
            ));
        }
        self.require_stack_len(count)?;
        let len = self.state.stack.len();
        self.state.stack[len - count..].rotate_left(1);
        Ok(())
    }

    pub fn pick(&mut self, depth: usize) -> Result<(), CalcError> {
        if depth == 0 {
            return Err(CalcError::InvalidInput(
                "pick depth must be at least 1".to_string(),
            ));
        }
        self.require_stack_len(depth)?;
        let len = self.state.stack.len();
        let value = self.state.stack[len - depth].clone();
        self.state.stack.push(value);
        Ok(())
    }

    pub fn pick_from_stack_index(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(2)?;
        let len = self.state.stack.len();
        let index = match self.state.stack.get(len - 1) {
            Some(Value::Real(v)) => Self::as_non_negative_integer(*v, "pick index")? as usize,
            Some(Value::Complex(c)) if c.im.abs() <= 1e-12 => {
                Self::as_non_negative_integer(c.re, "pick index")? as usize
            }
            _ => {
                return Err(CalcError::TypeMismatch(
                    "pick index must be a non-negative integer scalar".to_string(),
                ));
            }
        };

        if index >= len - 1 {
            return Err(CalcError::InvalidInput(format!(
                "pick index out of range: {index} (stack lines are 0..{})",
                len - 2
            )));
        }

        let value = self.state.stack[index].clone();
        self.state.stack[len - 1] = value;
        Ok(())
    }

    pub fn add(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_add(a, b)?)),
            (Value::Matrix(a), scalar) => {
                let scalar = Self::as_complex(scalar, "+")?;
                Ok(Value::Matrix(Self::matrix_scalar_add(a, scalar)))
            }
            (scalar, Value::Matrix(b)) => {
                let scalar = Self::as_complex(scalar, "+")?;
                Ok(Value::Matrix(Self::matrix_scalar_add(b, scalar)))
            }
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a + b)),
            _ => {
                let left = Self::as_complex(left, "+")?;
                let right = Self::as_complex(right, "+")?;
                Ok(Value::Complex(Complex {
                    re: left.re + right.re,
                    im: left.im + right.im,
                }))
            }
        })
    }

    pub fn sub(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_sub(a, b)?)),
            (Value::Matrix(a), scalar) => {
                let scalar = Self::as_complex(scalar, "-")?;
                Ok(Value::Matrix(Self::matrix_scalar_sub(a, scalar)))
            }
            (scalar, Value::Matrix(b)) => {
                let scalar = Self::as_complex(scalar, "-")?;
                Ok(Value::Matrix(Self::matrix_scalar_lsub(scalar, b)))
            }
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a - b)),
            _ => {
                let left = Self::as_complex(left, "-")?;
                let right = Self::as_complex(right, "-")?;
                Ok(Value::Complex(Complex {
                    re: left.re - right.re,
                    im: left.im - right.im,
                }))
            }
        })
    }

    pub fn mul(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_mul(a, b)?)),
            (Value::Matrix(a), scalar) => {
                let scalar = Self::as_complex(scalar, "*")?;
                Ok(Value::Matrix(Self::matrix_scalar_mul(a, scalar)))
            }
            (scalar, Value::Matrix(b)) => {
                let scalar = Self::as_complex(scalar, "*")?;
                Ok(Value::Matrix(Self::matrix_scalar_mul(b, scalar)))
            }
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a * b)),
            _ => {
                let left = Self::as_complex(left, "*")?;
                let right = Self::as_complex(right, "*")?;
                Ok(Value::Complex(Complex {
                    re: left.re * right.re - left.im * right.im,
                    im: left.re * right.im + left.im * right.re,
                }))
            }
        })
    }

    pub fn div(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(_), Value::Real(b)) if *b == 0.0 => Err(CalcError::DivideByZero),
            (Value::Matrix(a), scalar) => {
                let scalar = Self::as_complex(scalar, "/")?;
                Ok(Value::Matrix(Self::matrix_scalar_div(a, scalar)?))
            }
            (scalar, Value::Matrix(b)) => {
                let scalar = Self::as_complex(scalar, "/")?;
                Ok(Value::Matrix(Self::matrix_scalar_ldiv(scalar, b)?))
            }
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a / b)),
            _ => {
                let left = Self::as_complex(left, "/")?;
                let right = Self::as_complex(right, "/")?;
                let denom = right.re * right.re + right.im * right.im;
                if denom == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Complex {
                    re: (left.re * right.re + left.im * right.im) / denom,
                    im: (left.im * right.re - left.re * right.im) / denom,
                }))
            }
        })
    }

    pub fn hadamard_mul(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => {
                Ok(Value::Matrix(Self::matrix_hadamard_mul(a, b)?))
            }
            (Value::Matrix(a), scalar) => {
                let scalar = Self::as_complex(scalar, "HadMul")?;
                Ok(Value::Matrix(Self::matrix_scalar_mul(a, scalar)))
            }
            (scalar, Value::Matrix(b)) => {
                let scalar = Self::as_complex(scalar, "HadMul")?;
                Ok(Value::Matrix(Self::matrix_scalar_mul(b, scalar)))
            }
            _ => Err(CalcError::TypeMismatch(
                "HadMul requires a matrix with either another equal-shape matrix or a scalar"
                    .to_string(),
            )),
        })
    }

    pub fn hadamard_div(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => {
                Ok(Value::Matrix(Self::matrix_hadamard_div(a, b)?))
            }
            (Value::Matrix(a), scalar) => {
                let scalar = Self::as_complex(scalar, "HadDiv")?;
                Ok(Value::Matrix(Self::matrix_scalar_div(a, scalar)?))
            }
            (scalar, Value::Matrix(b)) => {
                let scalar = Self::as_complex(scalar, "HadDiv")?;
                Ok(Value::Matrix(Self::matrix_scalar_ldiv(scalar, b)?))
            }
            _ => Err(CalcError::TypeMismatch(
                "HadDiv requires a matrix with either another equal-shape matrix or a scalar"
                    .to_string(),
            )),
        })
    }

    pub fn sqrt(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) if *v < 0.0 => Err(CalcError::DomainError(
                "sqrt is undefined for negative real values".to_string(),
            )),
            Value::Real(v) => Ok(Value::Real(v.sqrt())),
            Value::Complex(c) => Ok(Value::Complex(Self::complex_sqrt(*c))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::complex_sqrt(entry))
                })?))
            }
        })
    }

    pub fn exp(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.exp())),
            Value::Complex(c) => Ok(Value::Complex(Self::complex_exp(*c))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::complex_exp(entry))
                })?))
            }
        })
    }

    pub fn ln(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) if *v <= 0.0 => Err(CalcError::DomainError(
                "ln is undefined for non-positive real values".to_string(),
            )),
            Value::Real(v) => Ok(Value::Real(v.ln())),
            Value::Complex(c) => Ok(Value::Complex(Self::complex_ln(*c))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).ln()))
                })?))
            }
        })
    }

    pub fn sin(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let radians = match mode {
                    AngleMode::Deg => v.to_radians(),
                    AngleMode::Rad => *v,
                };
                Ok(Value::Real(radians.sin()))
            }
            Value::Complex(c) => Ok(Value::Complex(Self::complex_sin(*c))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).sin()))
                })?))
            }
        })
    }

    pub fn cos(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let radians = match mode {
                    AngleMode::Deg => v.to_radians(),
                    AngleMode::Rad => *v,
                };
                Ok(Value::Real(radians.cos()))
            }
            Value::Complex(c) => Ok(Value::Complex(Self::complex_cos(*c))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).cos()))
                })?))
            }
        })
    }

    pub fn tan(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let radians = match mode {
                    AngleMode::Deg => v.to_radians(),
                    AngleMode::Rad => *v,
                };
                Ok(Value::Real(radians.tan()))
            }
            Value::Complex(c) => {
                let numerator = Self::complex_sin(*c);
                let denominator = Self::complex_cos(*c);
                let denom_norm = denominator.re * denominator.re + denominator.im * denominator.im;
                if denom_norm == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Complex {
                    re: (numerator.re * denominator.re + numerator.im * denominator.im)
                        / denom_norm,
                    im: (numerator.im * denominator.re - numerator.re * denominator.im)
                        / denom_norm,
                }))
            }
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).tan()))
                })?))
            }
        })
    }

    pub fn asin(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) if !(-1.0..=1.0).contains(v) => Err(CalcError::DomainError(
                "asin is undefined for real values outside [-1, 1]".to_string(),
            )),
            Value::Real(v) => {
                let radians = v.asin();
                let output = match mode {
                    AngleMode::Deg => radians.to_degrees(),
                    AngleMode::Rad => radians,
                };
                Ok(Value::Real(output))
            }
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).asin(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).asin()))
                })?))
            }
        })
    }

    pub fn acos(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) if !(-1.0..=1.0).contains(v) => Err(CalcError::DomainError(
                "acos is undefined for real values outside [-1, 1]".to_string(),
            )),
            Value::Real(v) => {
                let radians = v.acos();
                let output = match mode {
                    AngleMode::Deg => radians.to_degrees(),
                    AngleMode::Rad => radians,
                };
                Ok(Value::Real(output))
            }
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).acos(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).acos()))
                })?))
            }
        })
    }

    pub fn atan(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let radians = v.atan();
                let output = match mode {
                    AngleMode::Deg => radians.to_degrees(),
                    AngleMode::Rad => radians,
                };
                Ok(Value::Real(output))
            }
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).atan(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).atan()))
                })?))
            }
        })
    }

    pub fn sinh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.sinh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).sinh(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).sinh()))
                })?))
            }
        })
    }

    pub fn cosh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.cosh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).cosh(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).cosh()))
                })?))
            }
        })
    }

    pub fn tanh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.tanh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).tanh(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).tanh()))
                })?))
            }
        })
    }

    pub fn asinh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.asinh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).asinh(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).asinh()))
                })?))
            }
        })
    }

    pub fn acosh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) if *v < 1.0 => Err(CalcError::DomainError(
                "acosh is undefined for real values below 1".to_string(),
            )),
            Value::Real(v) => Ok(Value::Real(v.acosh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).acosh(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).acosh()))
                })?))
            }
        })
    }

    pub fn atanh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) if v.abs() >= 1.0 => Err(CalcError::DomainError(
                "atanh is undefined for real values with |x| >= 1".to_string(),
            )),
            Value::Real(v) => Ok(Value::Real(v.atanh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).atanh(),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).atanh()))
                })?))
            }
        })
    }

    pub fn log10(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) if *v <= 0.0 => Err(CalcError::DomainError(
                "log10 is undefined for non-positive real values".to_string(),
            )),
            Value::Real(v) => Ok(Value::Real(v.log10())),
            Value::Complex(c) => {
                let ln10 = Complex64::new(10.0, 0.0).ln();
                let out = Self::to_complex64(*c).ln() / ln10;
                Ok(Value::Complex(Self::from_complex64(out)))
            }
            Value::Matrix(matrix) => {
                let ln10 = Complex64::new(10.0, 0.0).ln();
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).ln() / ln10))
                })?))
            }
        })
    }

    pub fn gamma(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(Self::real_gamma(*v))),
            Value::Complex(_) => Err(CalcError::TypeMismatch(
                "gamma currently supports real values only".to_string(),
            )),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "gamma",
                |v| Ok(Self::real_gamma(v)),
            )?)),
        })
    }

    pub fn erf(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(Self::real_erf(*v))),
            Value::Complex(_) => Err(CalcError::TypeMismatch(
                "erf currently supports real values only".to_string(),
            )),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "erf",
                |v| Ok(Self::real_erf(v)),
            )?)),
        })
    }

    pub fn pow(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(_), Value::Matrix(_)) => {
                Self::matrix_elementwise_binary(left, right, "pow", |lhs, rhs| Ok(lhs.powc(rhs)))?
                    .ok_or_else(|| CalcError::TypeMismatch("pow requires operands".to_string()))
            }
            (Value::Matrix(a), scalar) => {
                let scalar = Self::as_complex(scalar, "pow")?;
                Ok(Value::Matrix(Self::matrix_scalar_pow(a, scalar)))
            }
            (scalar, Value::Matrix(b)) => {
                let scalar = Self::as_complex(scalar, "pow")?;
                Ok(Value::Matrix(Self::matrix_scalar_lpow(scalar, b)))
            }
            (Value::Real(base), Value::Real(exp)) => Ok(Value::Real(base.powf(*exp))),
            _ => {
                let left = Self::as_complex(left, "pow")?;
                let right = Self::as_complex(right, "pow")?;
                let out = Self::to_complex64(left).powc(Self::to_complex64(right));
                Ok(Value::Complex(Self::from_complex64(out)))
            }
        })
    }

    pub fn percent(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(_), _) | (_, Value::Matrix(_)) => {
                Self::matrix_elementwise_binary(left, right, "percent", |lhs, rhs| {
                    if lhs.im.abs() > 1e-12 || rhs.im.abs() > 1e-12 {
                        return Err(CalcError::TypeMismatch(
                            "percent requires real-valued operands".to_string(),
                        ));
                    }
                    Ok(Complex64::new(lhs.re * rhs.re / 100.0, 0.0))
                })?
                .ok_or_else(|| CalcError::TypeMismatch("percent requires operands".to_string()))
            }
            (Value::Real(base), Value::Real(percent)) => Ok(Value::Real(base * percent / 100.0)),
            _ => Err(CalcError::TypeMismatch(
                "percent currently supports real values only".to_string(),
            )),
        })
    }

    pub fn inv(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                if *v == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(1.0 / v))
            }
            Value::Complex(c) => {
                let denom = c.re * c.re + c.im * c.im;
                if denom == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Complex {
                    re: c.re / denom,
                    im: -c.im / denom,
                }))
            }
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    let denom = entry.re * entry.re + entry.im * entry.im;
                    if denom == 0.0 {
                        return Err(CalcError::DivideByZero);
                    }
                    Ok(Complex {
                        re: entry.re / denom,
                        im: -entry.im / denom,
                    })
                })?))
            }
        })
    }

    pub fn square(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v * v)),
            Value::Complex(c) => Ok(Value::Complex(Complex {
                re: c.re * c.re - c.im * c.im,
                im: 2.0 * c.re * c.im,
            })),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    let z = Self::to_complex64(entry);
                    Ok(Self::from_complex64(z * z))
                })?))
            }
        })
    }

    pub fn root(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| {
            if let Some(value) =
                Self::matrix_elementwise_binary(left, right, "root", |lhs, rhs| {
                    if rhs.norm() == 0.0 {
                        return Err(CalcError::DivideByZero);
                    }
                    Ok(lhs.powc(Complex64::new(1.0, 0.0) / rhs))
                })?
            {
                return Ok(value);
            }
            match (left, right) {
                (Value::Real(x), Value::Real(y)) => {
                    if *y == 0.0 {
                        return Err(CalcError::DivideByZero);
                    }
                    Ok(Value::Real(x.powf(1.0 / y)))
                }
                _ => {
                    let x = Self::as_complex(left, "root")?;
                    let y = Self::as_complex(right, "root")?;
                    let out = Self::to_complex64(x)
                        .powc(Complex64::new(1.0, 0.0) / Self::to_complex64(y));
                    Ok(Value::Complex(Self::from_complex64(out)))
                }
            }
        })
    }

    pub fn exp10(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(10.0_f64.powf(*v))),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Complex64::new(10.0, 0.0).powc(Self::to_complex64(*c)),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(
                        Complex64::new(10.0, 0.0).powc(Self::to_complex64(entry)),
                    ))
                })?))
            }
        })
    }

    pub fn exp2(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(2.0_f64.powf(*v))),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Complex64::new(2.0, 0.0).powc(Self::to_complex64(*c)),
            ))),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(
                        Complex64::new(2.0, 0.0).powc(Self::to_complex64(entry)),
                    ))
                })?))
            }
        })
    }

    pub fn log2(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) if *v <= 0.0 => Err(CalcError::DomainError(
                "log2 is undefined for non-positive real values".to_string(),
            )),
            Value::Real(v) => Ok(Value::Real(v.log2())),
            Value::Complex(c) => {
                let out = Self::to_complex64(*c).ln() / Complex64::new(2.0, 0.0).ln();
                Ok(Value::Complex(Self::from_complex64(out)))
            }
            Value::Matrix(matrix) => {
                let ln2 = Complex64::new(2.0, 0.0).ln();
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Self::from_complex64(Self::to_complex64(entry).ln() / ln2))
                })?))
            }
        })
    }

    pub fn signum(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.signum())),
            Value::Complex(c) => {
                let norm = (c.re * c.re + c.im * c.im).sqrt();
                if norm == 0.0 {
                    Ok(Value::Complex(Complex { re: 0.0, im: 0.0 }))
                } else {
                    Ok(Value::Complex(Complex {
                        re: c.re / norm,
                        im: c.im / norm,
                    }))
                }
            }
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    let norm = (entry.re * entry.re + entry.im * entry.im).sqrt();
                    if norm == 0.0 {
                        Ok(Complex { re: 0.0, im: 0.0 })
                    } else {
                        Ok(Complex {
                            re: entry.re / norm,
                            im: entry.im / norm,
                        })
                    }
                })?))
            }
        })
    }

    pub fn abs(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.abs())),
            Value::Complex(c) => Ok(Value::Real((c.re * c.re + c.im * c.im).sqrt())),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Complex {
                        re: (entry.re * entry.re + entry.im * entry.im).sqrt(),
                        im: 0.0,
                    })
                })?))
            }
        })
    }

    pub fn abs_sq(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v * v)),
            Value::Complex(c) => Ok(Value::Real(c.re * c.re + c.im * c.im)),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Complex {
                        re: entry.re * entry.re + entry.im * entry.im,
                        im: 0.0,
                    })
                })?))
            }
        })
    }

    pub fn arg(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Complex(c) => {
                let radians = c.im.atan2(c.re);
                let out = match mode {
                    AngleMode::Deg => radians.to_degrees(),
                    AngleMode::Rad => radians,
                };
                Ok(Value::Real(out))
            }
            Value::Real(v) => {
                let radians = if *v >= 0.0 { 0.0 } else { std::f64::consts::PI };
                let out = match mode {
                    AngleMode::Deg => radians.to_degrees(),
                    AngleMode::Rad => radians,
                };
                Ok(Value::Real(out))
            }
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    let radians = entry.im.atan2(entry.re);
                    let out = match mode {
                        AngleMode::Deg => radians.to_degrees(),
                        AngleMode::Rad => radians,
                    };
                    Ok(Complex { re: out, im: 0.0 })
                })?))
            }
        })
    }

    pub fn conjugate(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Complex(c) => Ok(Value::Complex(Complex {
                re: c.re,
                im: -c.im,
            })),
            Value::Real(v) => Ok(Value::Real(*v)),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_conjugate(matrix))),
        })
    }

    pub fn real_part(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(*v)),
            Value::Complex(c) => Ok(Value::Real(c.re)),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Complex {
                        re: entry.re,
                        im: 0.0,
                    })
                })?))
            }
        })
    }

    pub fn imag_part(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(_) => Ok(Value::Real(0.0)),
            Value::Complex(c) => Ok(Value::Real(c.im)),
            Value::Matrix(matrix) => {
                Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                    Ok(Complex {
                        re: entry.im,
                        im: 0.0,
                    })
                })?))
            }
        })
    }

    pub fn cart(&mut self) -> Result<(), CalcError> {
        self.complex_stack_transform(ComplexTransformMode::Cartesian)
    }

    pub fn pol(&mut self) -> Result<(), CalcError> {
        self.complex_stack_transform(ComplexTransformMode::Polar)
    }

    pub fn npol(&mut self) -> Result<(), CalcError> {
        self.complex_stack_transform(ComplexTransformMode::NormalizedPolar)
    }

    pub fn atan2(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_binary_op(|left, right| {
            if let Some(value) =
                Self::matrix_elementwise_binary(left, right, "atan2", |lhs, rhs| {
                    if lhs.im.abs() > 1e-12 || rhs.im.abs() > 1e-12 {
                        return Err(CalcError::TypeMismatch(
                            "atan2 requires real-valued operands".to_string(),
                        ));
                    }
                    let mut out = lhs.re.atan2(rhs.re);
                    if mode == AngleMode::Deg {
                        out = out.to_degrees();
                    }
                    Ok(Complex64::new(out, 0.0))
                })?
            {
                return Ok(value);
            }
            match (left, right) {
                (Value::Real(y), Value::Real(x)) => {
                    let mut out = y.atan2(*x);
                    if mode == AngleMode::Deg {
                        out = out.to_degrees();
                    }
                    Ok(Value::Real(out))
                }
                _ => Err(CalcError::TypeMismatch(
                    "atan2 requires two real operands (y then x)".to_string(),
                )),
            }
        })
    }

    pub fn to_rad(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.to_radians())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "to_rad",
                |v| Ok(v.to_radians()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "to_rad currently supports real values only".to_string(),
            )),
        })
    }

    pub fn to_deg(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.to_degrees())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "to_deg",
                |v| Ok(v.to_degrees()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "to_deg currently supports real values only".to_string(),
            )),
        })
    }

    pub fn factorial(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let n = Self::as_non_negative_integer(*v, "factorial")?;
                let mut out = 1.0;
                for i in 2..=n {
                    out *= i as f64;
                }
                Ok(Value::Real(out))
            }
            _ => Err(CalcError::TypeMismatch(
                "factorial currently supports real values only".to_string(),
            )),
        })
    }

    pub fn ncr(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(n), Value::Real(r)) => {
                let n = Self::as_non_negative_integer(*n, "nCr n")?;
                let r = Self::as_non_negative_integer(*r, "nCr r")?;
                if r > n {
                    return Err(CalcError::DomainError("nCr requires n >= r".to_string()));
                }
                Ok(Value::Real(Self::ncr_value(n, r)))
            }
            _ => Err(CalcError::TypeMismatch(
                "nCr currently supports real values only".to_string(),
            )),
        })
    }

    pub fn npr(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(n), Value::Real(r)) => {
                let n = Self::as_non_negative_integer(*n, "nPr n")?;
                let r = Self::as_non_negative_integer(*r, "nPr r")?;
                if r > n {
                    return Err(CalcError::DomainError("nPr requires n >= r".to_string()));
                }
                let mut out = 1.0;
                for i in 0..r {
                    out *= (n - i) as f64;
                }
                Ok(Value::Real(out))
            }
            _ => Err(CalcError::TypeMismatch(
                "nPr currently supports real values only".to_string(),
            )),
        })
    }

    pub fn modulo(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(x), Value::Real(y)) => {
                if *y == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(x.rem_euclid(*y)))
            }
            _ => Err(CalcError::TypeMismatch(
                "mod currently supports real values only".to_string(),
            )),
        })
    }

    pub fn rand_num(&mut self) -> Result<(), CalcError> {
        let next = self.next_random();
        self.state.stack.push(Value::Real(next));
        Ok(())
    }

    pub fn gcd(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(a), Value::Real(b)) => {
                let a = Self::as_integer(*a, "gcd a")?;
                let b = Self::as_integer(*b, "gcd b")?;
                let g = Self::gcd_u64(a.unsigned_abs(), b.unsigned_abs());
                Ok(Value::Real(g as f64))
            }
            _ => Err(CalcError::TypeMismatch(
                "gcd currently supports real values only".to_string(),
            )),
        })
    }

    pub fn lcm(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(a), Value::Real(b)) => {
                let a = Self::as_integer(*a, "lcm a")?;
                let b = Self::as_integer(*b, "lcm b")?;
                let a_abs = a.unsigned_abs();
                let b_abs = b.unsigned_abs();
                if a_abs == 0 || b_abs == 0 {
                    return Ok(Value::Real(0.0));
                }
                let g = Self::gcd_u64(a_abs, b_abs);
                let l = (a_abs / g).saturating_mul(b_abs);
                Ok(Value::Real(l as f64))
            }
            _ => Err(CalcError::TypeMismatch(
                "lcm currently supports real values only".to_string(),
            )),
        })
    }

    pub fn round_value(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.round())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "round",
                |v| Ok(v.round()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "rnd currently supports real values only".to_string(),
            )),
        })
    }

    pub fn floor_value(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.floor())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "floor",
                |v| Ok(v.floor()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "floor currently supports real values only".to_string(),
            )),
        })
    }

    pub fn ceil_value(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.ceil())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "ceil",
                |v| Ok(v.ceil()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "ceil currently supports real values only".to_string(),
            )),
        })
    }

    pub fn dec_part(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v - v.trunc())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "decP",
                |v| Ok(v - v.trunc()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "decP currently supports real values only".to_string(),
            )),
        })
    }

    pub fn memory_store(&mut self, register: usize) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let index = Self::validate_register(register)?;
        self.state.memory[index] = self.state.stack.last().cloned();
        Ok(())
    }

    pub fn memory_recall(&mut self, register: usize) -> Result<(), CalcError> {
        let index = Self::validate_register(register)?;
        let value = self.state.memory[index]
            .clone()
            .ok_or(CalcError::EmptyRegister(register))?;
        self.state.stack.push(value);
        Ok(())
    }

    pub fn memory_clear(&mut self, register: usize) -> Result<(), CalcError> {
        let index = Self::validate_register(register)?;
        self.state.memory[index] = None;
        Ok(())
    }

    pub fn transpose(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_transpose(matrix))),
            _ => Err(CalcError::TypeMismatch(
                "transpose requires a matrix value".to_string(),
            )),
        })
    }

    pub fn push_identity(&mut self, size: usize) -> Result<(), CalcError> {
        if size == 0 {
            return Err(CalcError::InvalidInput(
                "identity matrix size must be non-zero".to_string(),
            ));
        }
        self.state
            .stack
            .push(Value::Matrix(Self::matrix_identity(size)));
        Ok(())
    }

    pub fn stack_vec(&mut self) -> Result<(), CalcError> {
        if self.state.stack.is_empty() {
            return Err(CalcError::InvalidInput(
                "stack_vec requires at least one scalar value on the stack".to_string(),
            ));
        }

        let mut data = Vec::with_capacity(self.state.stack.len());
        for value in &self.state.stack {
            match value {
                Value::Real(v) => data.push(Complex { re: *v, im: 0.0 }),
                Value::Complex(c) => data.push(*c),
                Value::Matrix(_) => {
                    return Err(CalcError::TypeMismatch(
                        "stack_vec requires stack values to be real or complex scalars only"
                            .to_string(),
                    ));
                }
            }
        }

        let vector = Matrix::new(data.len(), 1, data)?;
        self.state.stack.clear();
        self.state.stack.push(Value::Matrix(vector));
        Ok(())
    }

    pub fn hstack(&mut self) -> Result<(), CalcError> {
        self.stack_combine(true)
    }

    pub fn vstack(&mut self) -> Result<(), CalcError> {
        self.stack_combine(false)
    }

    pub fn ravel(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        let matrix = match self.state.stack.get(len - 1) {
            Some(Value::Matrix(matrix)) => matrix.clone(),
            _ => {
                return Err(CalcError::TypeMismatch(
                    "ravel requires a matrix value".to_string(),
                ));
            }
        };

        if matrix.rows == 1 || matrix.cols == 1 {
            self.state.stack.truncate(len - 1);
            for entry in matrix.data {
                if entry.im.abs() <= 1e-12 {
                    self.state.stack.push(Value::Real(entry.re));
                } else {
                    self.state.stack.push(Value::Complex(entry));
                }
            }
            Ok(())
        } else {
            let vector = Matrix::new(matrix.rows * matrix.cols, 1, matrix.data)?;
            self.state.stack[len - 1] = Value::Matrix(vector);
            Ok(())
        }
    }

    pub fn hravel(&mut self) -> Result<(), CalcError> {
        self.matrix_ravel(true)
    }

    pub fn vravel(&mut self) -> Result<(), CalcError> {
        self.matrix_ravel(false)
    }

    pub fn determinant(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Complex(Self::matrix_determinant(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "determinant requires a matrix value".to_string(),
            )),
        })
    }

    pub fn inverse(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_inverse(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "inverse requires a matrix value".to_string(),
            )),
        })
    }

    pub fn solve_ax_b(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_solve(a, b)?)),
            _ => Err(CalcError::TypeMismatch(
                "solve_ax_b requires two matrix operands (A then B)".to_string(),
            )),
        })
    }

    pub fn solve_lstsq(&mut self) -> Result<Option<String>, CalcError> {
        self.require_stack_len(2)?;
        let len = self.state.stack.len();
        let (a, b) = match (self.state.stack.get(len - 2), self.state.stack.get(len - 1)) {
            (Some(Value::Matrix(a)), Some(Value::Matrix(b))) => (a.clone(), b.clone()),
            _ => {
                return Err(CalcError::TypeMismatch(
                    "solve_lstsq requires two matrix operands (A then B)".to_string(),
                ));
            }
        };

        let (x, warning) = Self::matrix_solve_lstsq(&a, &b)?;
        self.state.stack.truncate(len - 2);
        self.state.stack.push(Value::Matrix(x));
        Ok(warning)
    }

    pub fn dot(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Complex(Self::matrix_dot(a, b)?)),
            _ => Err(CalcError::TypeMismatch(
                "dot requires two vector matrices".to_string(),
            )),
        })
    }

    pub fn cross(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_cross(a, b)?)),
            _ => Err(CalcError::TypeMismatch(
                "cross requires two 3-element vector matrices".to_string(),
            )),
        })
    }

    pub fn trace(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Complex(Self::matrix_trace(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "trace requires a matrix value".to_string(),
            )),
        })
    }

    pub fn norm_p(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(matrix), Value::Real(p)) => {
                Ok(Value::Real(Self::matrix_p_norm(matrix, *p)?))
            }
            (Value::Matrix(matrix), Value::Complex(p)) if p.im.abs() <= 1e-12 => {
                Ok(Value::Real(Self::matrix_p_norm(matrix, p.re)?))
            }
            _ => Err(CalcError::TypeMismatch(
                "norm_p requires a matrix followed by a real p value".to_string(),
            )),
        })
    }

    pub fn diag(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_diag(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "diag requires a vector matrix value".to_string(),
            )),
        })
    }

    pub fn toep(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_toeplitz(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "toep requires a vector matrix value".to_string(),
            )),
        })
    }

    pub fn mat_exp(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_exp(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "MatExp requires a matrix value".to_string(),
            )),
        })
    }

    pub fn hermitian(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_hermitian(matrix))),
            _ => Err(CalcError::TypeMismatch(
                "Hermitian requires a matrix value".to_string(),
            )),
        })
    }

    pub fn mat_pow(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(matrix), Value::Real(exp)) => {
                let power = Self::as_integer(*exp, "matrix power")?;
                Ok(Value::Matrix(Self::matrix_mat_pow(matrix, power)?))
            }
            (Value::Matrix(matrix), Value::Complex(exp)) if exp.im.abs() <= 1e-12 => {
                let power = Self::as_integer(exp.re, "matrix power")?;
                Ok(Value::Matrix(Self::matrix_mat_pow(matrix, power)?))
            }
            _ => Err(CalcError::TypeMismatch(
                "MatPow requires a matrix followed by an integer exponent".to_string(),
            )),
        })
    }

    pub fn qr(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        let matrix = match self.state.stack.get(len - 1) {
            Some(Value::Matrix(matrix)) => matrix.clone(),
            _ => {
                return Err(CalcError::TypeMismatch(
                    "QR requires a matrix value".to_string(),
                ));
            }
        };

        let (q, r) = Self::matrix_qr(&matrix)?;
        self.state.stack[len - 1] = Value::Matrix(q);
        self.state.stack.push(Value::Matrix(r));
        Ok(())
    }

    pub fn lu(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        let matrix = match self.state.stack.get(len - 1) {
            Some(Value::Matrix(matrix)) => matrix.clone(),
            _ => {
                return Err(CalcError::TypeMismatch(
                    "LU requires a matrix value".to_string(),
                ));
            }
        };

        let (p, l, u) = Self::matrix_lu(&matrix)?;
        self.state.stack[len - 1] = Value::Matrix(p);
        self.state.stack.push(Value::Matrix(l));
        self.state.stack.push(Value::Matrix(u));
        Ok(())
    }

    pub fn svd(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        let matrix = match self.state.stack.get(len - 1) {
            Some(Value::Matrix(matrix)) => matrix.clone(),
            _ => {
                return Err(CalcError::TypeMismatch(
                    "SVD requires a matrix value".to_string(),
                ));
            }
        };

        let (u, s, vt) = Self::matrix_svd(&matrix)?;
        self.state.stack[len - 1] = Value::Matrix(u);
        self.state.stack.push(Value::Matrix(s));
        self.state.stack.push(Value::Matrix(vt));
        Ok(())
    }

    pub fn evd(&mut self) -> Result<Option<String>, CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        let matrix = match self.state.stack.get(len - 1) {
            Some(Value::Matrix(matrix)) => matrix.clone(),
            _ => {
                return Err(CalcError::TypeMismatch(
                    "EVD requires a matrix value".to_string(),
                ));
            }
        };

        let (v, d, warning) = Self::matrix_evd(&matrix)?;
        self.state.stack[len - 1] = Value::Matrix(v);
        self.state.stack.push(Value::Matrix(d));
        Ok(warning)
    }

    pub fn mean(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_mean, "mean")
    }

    pub fn mode(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_mode, "mode")
    }

    pub fn variance(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_variance, "variance")
    }

    pub fn std_dev(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_std_dev, "std_dev")
    }

    pub fn max_value(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_max, "max")
    }

    pub fn min_value(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_min, "min")
    }

    fn apply_stat_op(
        &mut self,
        op: fn(&Matrix) -> Result<f64, CalcError>,
        label: &str,
    ) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();

        match self.state.stack.last() {
            Some(Value::Matrix(matrix)) => {
                let result = op(matrix)?;
                self.state.stack[len - 1] = Value::Real(result);
            }
            Some(_) => {
                let scalar_vector = Self::stack_real_scalars_as_vector(&self.state.stack, label)?;
                let result = op(&scalar_vector)?;
                self.state.stack.clear();
                self.state.stack.push(Value::Real(result));
            }
            None => unreachable!("prechecked non-empty stack"),
        }

        Ok(())
    }

    fn stack_real_scalars_as_vector(stack: &[Value], label: &str) -> Result<Matrix, CalcError> {
        let mut data = Vec::with_capacity(stack.len());
        for value in stack {
            match value {
                Value::Real(v) => data.push(Complex { re: *v, im: 0.0 }),
                Value::Complex(c) if c.im.abs() <= 1e-12 => {
                    data.push(Complex { re: c.re, im: 0.0 })
                }
                Value::Complex(_) => {
                    return Err(CalcError::TypeMismatch(format!(
                        "{label} over scalar stack requires real-valued scalars"
                    )));
                }
                Value::Matrix(_) => {
                    return Err(CalcError::TypeMismatch(format!(
                        "{label} requires a vector matrix or a scalar-only stack"
                    )));
                }
            }
        }

        Matrix::new(1, data.len(), data)
    }

    fn require_stack_len(&self, needed: usize) -> Result<(), CalcError> {
        let available = self.state.stack.len();
        if available < needed {
            return Err(CalcError::StackUnderflow { needed, available });
        }

        Ok(())
    }

    fn apply_unary_op<F>(&mut self, op: F) -> Result<(), CalcError>
    where
        F: Fn(&Value) -> Result<Value, CalcError>,
    {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        let value = self
            .state
            .stack
            .get(len - 1)
            .expect("prechecked stack length");
        let result = op(value)?;
        self.state.stack[len - 1] = result;
        Ok(())
    }

    fn apply_binary_op<F>(&mut self, op: F) -> Result<(), CalcError>
    where
        F: Fn(&Value, &Value) -> Result<Value, CalcError>,
    {
        self.require_stack_len(2)?;
        let len = self.state.stack.len();
        let left = self
            .state
            .stack
            .get(len - 2)
            .expect("prechecked stack length");
        let right = self
            .state
            .stack
            .get(len - 1)
            .expect("prechecked stack length");
        let result = op(left, right)?;
        self.state.stack.truncate(len - 2);
        self.state.stack.push(result);
        Ok(())
    }

    fn stack_count_value(value: &Value, label: &str) -> Result<usize, CalcError> {
        let count = match value {
            Value::Real(v) => Self::as_non_negative_integer(*v, label)?,
            Value::Complex(c) if c.im.abs() <= 1e-12 => Self::as_non_negative_integer(c.re, label)?,
            _ => {
                return Err(CalcError::TypeMismatch(format!(
                    "{label} must be a non-negative integer scalar"
                )));
            }
        };
        usize::try_from(count)
            .map_err(|_| CalcError::InvalidInput(format!("{label} is too large for this platform")))
    }

    fn stack_combine(&mut self, horizontal: bool) -> Result<(), CalcError> {
        self.require_stack_len(2)?;
        let len = self.state.stack.len();
        let count = Self::stack_count_value(
            self.state
                .stack
                .get(len - 1)
                .expect("prechecked count value"),
            if horizontal {
                "hstack count"
            } else {
                "vstack count"
            },
        )?;
        if count == 0 {
            return Err(CalcError::InvalidInput(
                "stack combine count must be at least 1".to_string(),
            ));
        }
        if len - 1 < count {
            return Err(CalcError::StackUnderflow {
                needed: count + 1,
                available: len,
            });
        }

        let start = len - 1 - count;
        let values = self.state.stack[start..len - 1].to_vec();
        let result = if values
            .iter()
            .all(|value| matches!(value, Value::Real(_) | Value::Complex(_)))
        {
            let data = values
                .iter()
                .map(|value| match value {
                    Value::Real(v) => Complex { re: *v, im: 0.0 },
                    Value::Complex(c) => *c,
                    Value::Matrix(_) => unreachable!("scalar prechecked"),
                })
                .collect::<Vec<_>>();
            if horizontal {
                Matrix::new(1, data.len(), data)?
            } else {
                Matrix::new(data.len(), 1, data)?
            }
        } else if values.iter().all(|value| matches!(value, Value::Matrix(_))) {
            let matrices = values
                .iter()
                .map(|value| match value {
                    Value::Matrix(matrix) => matrix,
                    _ => unreachable!("matrix prechecked"),
                })
                .collect::<Vec<_>>();
            let first = matrices[0];
            if matrices
                .iter()
                .any(|matrix| matrix.rows != first.rows || matrix.cols != first.cols)
            {
                return Err(CalcError::DimensionMismatch {
                    expected: first.rows * first.cols,
                    actual: matrices
                        .iter()
                        .find(|matrix| matrix.rows != first.rows || matrix.cols != first.cols)
                        .map(|matrix| matrix.rows * matrix.cols)
                        .unwrap_or(first.rows * first.cols),
                });
            }

            if horizontal {
                let mut data = Vec::with_capacity(first.rows * first.cols * matrices.len());
                for row in 0..first.rows {
                    for matrix in &matrices {
                        let row_start = row * first.cols;
                        let row_end = row_start + first.cols;
                        data.extend_from_slice(&matrix.data[row_start..row_end]);
                    }
                }
                Matrix::new(first.rows, first.cols * matrices.len(), data)?
            } else {
                let mut data = Vec::with_capacity(first.rows * first.cols * matrices.len());
                for matrix in &matrices {
                    data.extend_from_slice(&matrix.data);
                }
                Matrix::new(first.rows * matrices.len(), first.cols, data)?
            }
        } else {
            return Err(CalcError::TypeMismatch(
                "stack combine values must all be scalars or all be matrices".to_string(),
            ));
        };

        self.state.stack.truncate(start);
        self.state.stack.push(Value::Matrix(result));
        Ok(())
    }

    fn matrix_ravel(&mut self, horizontal: bool) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        let matrix = match self.state.stack.get(len - 1) {
            Some(Value::Matrix(matrix)) => matrix.clone(),
            _ => {
                return Err(CalcError::TypeMismatch(
                    "ravel requires a matrix value".to_string(),
                ));
            }
        };
        self.state.stack.truncate(len - 1);

        if matrix.rows == 1 || matrix.cols == 1 {
            for entry in matrix.data {
                if entry.im.abs() <= 1e-12 {
                    self.state.stack.push(Value::Real(entry.re));
                } else {
                    self.state.stack.push(Value::Complex(entry));
                }
            }
            return Ok(());
        }

        if horizontal {
            for col in 0..matrix.cols {
                let mut data = Vec::with_capacity(matrix.rows);
                for row in 0..matrix.rows {
                    data.push(matrix.data[row * matrix.cols + col]);
                }
                self.state
                    .stack
                    .push(Value::Matrix(Matrix::new(matrix.rows, 1, data)?));
            }
        } else {
            for row in 0..matrix.rows {
                let start = row * matrix.cols;
                let end = start + matrix.cols;
                let data = matrix.data[start..end].to_vec();
                self.state
                    .stack
                    .push(Value::Matrix(Matrix::new(1, matrix.cols, data)?));
            }
        }
        Ok(())
    }

    fn map_matrix_entries<F>(matrix: &Matrix, mut op: F) -> Result<Matrix, CalcError>
    where
        F: FnMut(Complex) -> Result<Complex, CalcError>,
    {
        let mut data = Vec::with_capacity(matrix.data.len());
        for value in &matrix.data {
            data.push(op(*value)?);
        }
        Matrix::new(matrix.rows, matrix.cols, data)
    }

    fn map_matrix_real_entries<F>(
        matrix: &Matrix,
        op_name: &str,
        mut op: F,
    ) -> Result<Matrix, CalcError>
    where
        F: FnMut(f64) -> Result<f64, CalcError>,
    {
        let mut data = Vec::with_capacity(matrix.data.len());
        for value in &matrix.data {
            if value.im.abs() > 1e-12 {
                return Err(CalcError::TypeMismatch(format!(
                    "{op_name} requires real-valued matrix entries"
                )));
            }
            data.push(Complex {
                re: op(value.re)?,
                im: 0.0,
            });
        }
        Matrix::new(matrix.rows, matrix.cols, data)
    }

    fn matrix_elementwise_binary<F>(
        left: &Value,
        right: &Value,
        op_name: &str,
        mut op: F,
    ) -> Result<Option<Value>, CalcError>
    where
        F: FnMut(Complex64, Complex64) -> Result<Complex64, CalcError>,
    {
        match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => {
                Self::require_same_shape(a, b, op_name)?;
                let mut data = Vec::with_capacity(a.data.len());
                for (lhs, rhs) in a.data.iter().zip(&b.data) {
                    data.push(Self::from_complex64(op(
                        Self::to_complex64(*lhs),
                        Self::to_complex64(*rhs),
                    )?));
                }
                Ok(Some(Value::Matrix(Matrix::new(a.rows, a.cols, data)?)))
            }
            (Value::Matrix(a), scalar) => {
                let rhs = Self::to_complex64(Self::as_complex(scalar, op_name)?);
                let mut data = Vec::with_capacity(a.data.len());
                for lhs in &a.data {
                    data.push(Self::from_complex64(op(Self::to_complex64(*lhs), rhs)?));
                }
                Ok(Some(Value::Matrix(Matrix::new(a.rows, a.cols, data)?)))
            }
            (scalar, Value::Matrix(b)) => {
                let lhs = Self::to_complex64(Self::as_complex(scalar, op_name)?);
                let mut data = Vec::with_capacity(b.data.len());
                for rhs in &b.data {
                    data.push(Self::from_complex64(op(lhs, Self::to_complex64(*rhs))?));
                }
                Ok(Some(Value::Matrix(Matrix::new(b.rows, b.cols, data)?)))
            }
            _ => Ok(None),
        }
    }

    fn as_complex(value: &Value, op: &str) -> Result<Complex, CalcError> {
        match value {
            Value::Real(v) => Ok(Complex { re: *v, im: 0.0 }),
            Value::Complex(c) => Ok(*c),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(format!(
                "{op} does not support matrix values"
            ))),
        }
    }

    fn as_real_scalar(value: &Value, label: &str) -> Result<f64, CalcError> {
        match value {
            Value::Real(v) => Ok(*v),
            Value::Complex(c) if c.im.abs() <= 1e-12 => Ok(c.re),
            Value::Complex(_) => Err(CalcError::TypeMismatch(format!(
                "{label} requires real scalar inputs"
            ))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(format!(
                "{label} requires scalar inputs"
            ))),
        }
    }

    fn complex_stack_transform(&mut self, mode: ComplexTransformMode) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        let angle_mode = self.state.angle_mode;
        if let Some(Value::Complex(c)) = self.state.stack.get(len - 1) {
            let (a, b) = match mode {
                ComplexTransformMode::Cartesian => (c.re, c.im),
                ComplexTransformMode::Polar => {
                    let mag = (c.re * c.re + c.im * c.im).sqrt();
                    let mut arg = c.im.atan2(c.re);
                    if angle_mode == AngleMode::Deg {
                        arg = arg.to_degrees();
                    }
                    (mag, arg)
                }
                ComplexTransformMode::NormalizedPolar => {
                    let mag = (c.re * c.re + c.im * c.im).sqrt();
                    let arg_cycles = c.im.atan2(c.re) / (2.0 * std::f64::consts::PI);
                    (mag, arg_cycles)
                }
            };
            self.state.stack[len - 1] = Value::Real(a);
            self.state.stack.push(Value::Real(b));
            return Ok(());
        }

        self.require_stack_len(2)?;
        let len = self.state.stack.len();
        let a = Self::as_real_scalar(
            self.state.stack.get(len - 2).expect("prechecked length"),
            "complex conversion",
        )?;
        let b = Self::as_real_scalar(
            self.state.stack.get(len - 1).expect("prechecked length"),
            "complex conversion",
        )?;
        let (re, im) = match mode {
            ComplexTransformMode::Cartesian => (a, b),
            ComplexTransformMode::Polar => {
                let theta = if angle_mode == AngleMode::Deg {
                    b.to_radians()
                } else {
                    b
                };
                (a * theta.cos(), a * theta.sin())
            }
            ComplexTransformMode::NormalizedPolar => {
                let theta = 2.0 * std::f64::consts::PI * b;
                (a * theta.cos(), a * theta.sin())
            }
        };
        self.state.stack.truncate(len - 2);
        self.state.stack.push(Value::Complex(Complex { re, im }));
        Ok(())
    }

    fn complex_sqrt(value: Complex) -> Complex {
        let magnitude = (value.re * value.re + value.im * value.im).sqrt();
        let real = ((magnitude + value.re) / 2.0).sqrt();
        let imag_sign = if value.im < 0.0 { -1.0 } else { 1.0 };
        let imag = imag_sign * ((magnitude - value.re) / 2.0).sqrt();
        Complex { re: real, im: imag }
    }

    fn complex_exp(value: Complex) -> Complex {
        let exp_real = value.re.exp();
        Complex {
            re: exp_real * value.im.cos(),
            im: exp_real * value.im.sin(),
        }
    }

    fn complex_ln(value: Complex) -> Complex {
        let modulus = (value.re * value.re + value.im * value.im).sqrt();
        let argument = value.im.atan2(value.re);
        Complex {
            re: modulus.ln(),
            im: argument,
        }
    }

    fn complex_sin(value: Complex) -> Complex {
        Complex {
            re: value.re.sin() * value.im.cosh(),
            im: value.re.cos() * value.im.sinh(),
        }
    }

    fn complex_cos(value: Complex) -> Complex {
        Complex {
            re: value.re.cos() * value.im.cosh(),
            im: -value.re.sin() * value.im.sinh(),
        }
    }

    fn to_complex64(value: Complex) -> Complex64 {
        Complex64::new(value.re, value.im)
    }

    fn from_complex64(value: Complex64) -> Complex {
        Complex {
            re: value.re,
            im: value.im,
        }
    }

    fn validate_register(register: usize) -> Result<usize, CalcError> {
        if register >= 26 {
            return Err(CalcError::InvalidRegister(register));
        }
        Ok(register)
    }

    fn as_integer(value: f64, label: &str) -> Result<i64, CalcError> {
        if !value.is_finite() {
            return Err(CalcError::InvalidInput(format!("{label} must be finite")));
        }
        if value.fract() != 0.0 {
            return Err(CalcError::InvalidInput(format!(
                "{label} must be an integer"
            )));
        }
        if value < i64::MIN as f64 || value > i64::MAX as f64 {
            return Err(CalcError::InvalidInput(format!("{label} is out of range")));
        }
        Ok(value as i64)
    }

    fn as_non_negative_integer(value: f64, label: &str) -> Result<u64, CalcError> {
        let int = Self::as_integer(value, label)?;
        if int < 0 {
            return Err(CalcError::DomainError(format!(
                "{label} must be non-negative"
            )));
        }
        Ok(int as u64)
    }

    fn ncr_value(n: u64, r: u64) -> f64 {
        if r == 0 || r == n {
            return 1.0;
        }
        let k = r.min(n - r);
        let mut out = 1.0;
        for i in 1..=k {
            out *= (n - k + i) as f64;
            out /= i as f64;
        }
        out
    }

    fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a
    }

    fn next_random(&mut self) -> f64 {
        self.state.rng_state = self
            .state
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let bits = self.state.rng_state >> 11;
        (bits as f64) / ((1u64 << 53) as f64)
    }

    fn real_erf(x: f64) -> f64 {
        // Abramowitz-Stegun 7.1.26 approximation.
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let t = 1.0 / (1.0 + 0.3275911 * x);
        let a1 = 0.254_829_592;
        let a2 = -0.284_496_736;
        let a3 = 1.421_413_741;
        let a4 = -1.453_152_027;
        let a5 = 1.061_405_429;
        let poly = ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t;
        sign * (1.0 - poly * (-x * x).exp())
    }

    fn real_gamma(z: f64) -> f64 {
        // Lanczos approximation with reflection formula.
        if z < 0.5 {
            let pi = std::f64::consts::PI;
            return pi / ((pi * z).sin() * Self::real_gamma(1.0 - z));
        }

        let p: [f64; 9] = [
            0.999_999_999_999_809_9,
            676.520_368_121_885_1,
            -1_259.139_216_722_402_8,
            771.323_428_777_653_1,
            -176.615_029_162_140_6,
            12.507_343_278_686_905,
            -0.138_571_095_265_720_12,
            0.000_009_984_369_578_019_572,
            0.000_000_150_563_273_514_931_16,
        ];
        let g = 7.0;
        let mut x = p[0];
        let zm1 = z - 1.0;
        for (i, coeff) in p.iter().enumerate().skip(1) {
            x += coeff / (zm1 + i as f64);
        }
        let t = zm1 + g + 0.5;
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(zm1 + 0.5) * (-t).exp() * x
    }

    fn matrix_add(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_same_shape(a, b, "matrix add")?;
        let data = a
            .data
            .iter()
            .zip(&b.data)
            .map(|(lhs, rhs)| {
                Self::from_complex64(Self::to_complex64(*lhs) + Self::to_complex64(*rhs))
            })
            .collect::<Vec<_>>();
        Matrix::new(a.rows, a.cols, data)
    }

    fn matrix_sub(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_same_shape(a, b, "matrix sub")?;
        let data = a
            .data
            .iter()
            .zip(&b.data)
            .map(|(lhs, rhs)| {
                Self::from_complex64(Self::to_complex64(*lhs) - Self::to_complex64(*rhs))
            })
            .collect::<Vec<_>>();
        Matrix::new(a.rows, a.cols, data)
    }

    fn matrix_mul(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        if a.cols != b.rows {
            return Err(CalcError::DimensionMismatch {
                expected: a.cols,
                actual: b.rows,
            });
        }

        let mut out = vec![Complex { re: 0.0, im: 0.0 }; a.rows * b.cols];
        for row in 0..a.rows {
            for col in 0..b.cols {
                let mut acc = Complex64::new(0.0, 0.0);
                for k in 0..a.cols {
                    let lhs = Self::to_complex64(a.data[row * a.cols + k]);
                    let rhs = Self::to_complex64(b.data[k * b.cols + col]);
                    acc += lhs * rhs;
                }
                out[row * b.cols + col] = Self::from_complex64(acc);
            }
        }
        Matrix::new(a.rows, b.cols, out)
    }

    fn matrix_hadamard_mul(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_same_shape(a, b, "HadMul")?;
        let data = a
            .data
            .iter()
            .zip(&b.data)
            .map(|(lhs, rhs)| {
                Self::from_complex64(Self::to_complex64(*lhs) * Self::to_complex64(*rhs))
            })
            .collect::<Vec<_>>();
        Matrix::new(a.rows, a.cols, data)
    }

    fn matrix_hadamard_div(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_same_shape(a, b, "HadDiv")?;
        let mut data = Vec::with_capacity(a.data.len());
        for (lhs, rhs) in a.data.iter().zip(&b.data) {
            let denom = Self::to_complex64(*rhs);
            if denom.norm() == 0.0 {
                return Err(CalcError::DivideByZero);
            }
            data.push(Self::from_complex64(Self::to_complex64(*lhs) / denom));
        }
        Matrix::new(a.rows, a.cols, data)
    }

    fn matrix_scalar_mul(matrix: &Matrix, scalar: Complex) -> Matrix {
        let scalar = Self::to_complex64(scalar);
        let data = matrix
            .data
            .iter()
            .map(|value| Self::from_complex64(Self::to_complex64(*value) * scalar))
            .collect();
        Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        }
    }

    fn matrix_scalar_add(matrix: &Matrix, scalar: Complex) -> Matrix {
        let scalar = Self::to_complex64(scalar);
        let data = matrix
            .data
            .iter()
            .map(|value| Self::from_complex64(Self::to_complex64(*value) + scalar))
            .collect();
        Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        }
    }

    fn matrix_scalar_sub(matrix: &Matrix, scalar: Complex) -> Matrix {
        let scalar = Self::to_complex64(scalar);
        let data = matrix
            .data
            .iter()
            .map(|value| Self::from_complex64(Self::to_complex64(*value) - scalar))
            .collect();
        Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        }
    }

    fn matrix_scalar_lsub(scalar: Complex, matrix: &Matrix) -> Matrix {
        let scalar = Self::to_complex64(scalar);
        let data = matrix
            .data
            .iter()
            .map(|value| Self::from_complex64(scalar - Self::to_complex64(*value)))
            .collect();
        Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        }
    }

    fn matrix_scalar_div(matrix: &Matrix, scalar: Complex) -> Result<Matrix, CalcError> {
        let scalar = Self::to_complex64(scalar);
        if scalar.norm() == 0.0 {
            return Err(CalcError::DivideByZero);
        }
        let data = matrix
            .data
            .iter()
            .map(|value| Self::from_complex64(Self::to_complex64(*value) / scalar))
            .collect();
        Ok(Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        })
    }

    fn matrix_scalar_ldiv(scalar: Complex, matrix: &Matrix) -> Result<Matrix, CalcError> {
        let scalar = Self::to_complex64(scalar);
        let mut data = Vec::with_capacity(matrix.data.len());
        for value in &matrix.data {
            let denom = Self::to_complex64(*value);
            if denom.norm() == 0.0 {
                return Err(CalcError::DivideByZero);
            }
            data.push(Self::from_complex64(scalar / denom));
        }
        Ok(Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        })
    }

    fn matrix_scalar_pow(matrix: &Matrix, scalar: Complex) -> Matrix {
        let scalar = Self::to_complex64(scalar);
        let data = matrix
            .data
            .iter()
            .map(|value| Self::from_complex64(Self::to_complex64(*value).powc(scalar)))
            .collect();
        Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        }
    }

    fn matrix_scalar_lpow(scalar: Complex, matrix: &Matrix) -> Matrix {
        let scalar = Self::to_complex64(scalar);
        let data = matrix
            .data
            .iter()
            .map(|value| Self::from_complex64(scalar.powc(Self::to_complex64(*value))))
            .collect();
        Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        }
    }

    fn matrix_conjugate(matrix: &Matrix) -> Matrix {
        let data = matrix
            .data
            .iter()
            .map(|value| Complex {
                re: value.re,
                im: -value.im,
            })
            .collect();
        Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        }
    }

    fn matrix_to_dmatrix(matrix: &Matrix) -> DMatrix<Complex64> {
        let data = matrix
            .data
            .iter()
            .map(|value| Self::to_complex64(*value))
            .collect::<Vec<_>>();
        DMatrix::from_row_slice(matrix.rows, matrix.cols, &data)
    }

    fn matrix_vector(matrix: &Matrix) -> Result<DVector<Complex64>, CalcError> {
        if matrix.rows == 1 || matrix.cols == 1 {
            let data = matrix
                .data
                .iter()
                .map(|value| Self::to_complex64(*value))
                .collect::<Vec<_>>();
            Ok(DVector::from_vec(data))
        } else {
            Err(CalcError::TypeMismatch(format!(
                "expected vector shape Nx1 or 1xN, got {}x{}",
                matrix.rows, matrix.cols
            )))
        }
    }

    fn matrix_dot(a: &Matrix, b: &Matrix) -> Result<Complex, CalcError> {
        let a_vec = Self::matrix_vector(a)?;
        let b_vec = Self::matrix_vector(b)?;
        if a_vec.len() != b_vec.len() {
            return Err(CalcError::DimensionMismatch {
                expected: a_vec.len(),
                actual: b_vec.len(),
            });
        }
        Ok(Self::from_complex64(a_vec.dotc(&b_vec)))
    }

    fn matrix_cross(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        let a_vec = Self::matrix_vector(a)?;
        let b_vec = Self::matrix_vector(b)?;
        if a_vec.len() != 3 || b_vec.len() != 3 {
            return Err(CalcError::TypeMismatch(
                "cross requires two vectors with exactly 3 elements".to_string(),
            ));
        }

        let av = Vector3::new(a_vec[0], a_vec[1], a_vec[2]);
        let bv = Vector3::new(b_vec[0], b_vec[1], b_vec[2]);
        let cv = av.cross(&bv);

        let as_row = a.rows == 1;
        let data = vec![
            Self::from_complex64(cv[0]),
            Self::from_complex64(cv[1]),
            Self::from_complex64(cv[2]),
        ];
        if as_row {
            Matrix::new(1, 3, data)
        } else {
            Matrix::new(3, 1, data)
        }
    }

    fn matrix_trace(matrix: &Matrix) -> Result<Complex, CalcError> {
        Self::require_square(matrix, "trace")?;
        Ok(Self::from_complex64(
            Self::matrix_to_dmatrix(matrix).trace(),
        ))
    }

    fn matrix_p_norm(matrix: &Matrix, p: f64) -> Result<f64, CalcError> {
        if !p.is_finite() || p <= 0.0 {
            return Err(CalcError::DomainError(
                "norm_p requires finite p > 0".to_string(),
            ));
        }

        let sum = matrix
            .data
            .iter()
            .map(|value| Self::to_complex64(*value).norm().powf(p))
            .sum::<f64>();
        Ok(sum.powf(1.0 / p))
    }

    fn matrix_diag(matrix: &Matrix) -> Result<Matrix, CalcError> {
        let vector = Self::matrix_vector(matrix)?;
        let n = vector.len();
        let mut data = vec![Complex { re: 0.0, im: 0.0 }; n * n];
        for i in 0..n {
            data[i * n + i] = Self::from_complex64(vector[i]);
        }
        Matrix::new(n, n, data)
    }

    fn matrix_toeplitz(matrix: &Matrix) -> Result<Matrix, CalcError> {
        let vector = Self::matrix_vector(matrix)?;
        let n = vector.len();
        let mut data = Vec::with_capacity(n * n);
        for row in 0..n {
            for col in 0..n {
                data.push(Self::from_complex64(vector[row.abs_diff(col)]));
            }
        }
        Matrix::new(n, n, data)
    }

    fn matrix_max_abs_entry(matrix: &DMatrix<Complex64>) -> f64 {
        matrix
            .iter()
            .map(|value| value.norm())
            .fold(0.0_f64, f64::max)
    }

    fn matrix_exp(matrix: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_square(matrix, "MatExp")?;

        let a = Self::matrix_to_dmatrix(matrix);
        let n = a.nrows();
        let mut result = DMatrix::<Complex64>::identity(n, n);
        let mut term = DMatrix::<Complex64>::identity(n, n);

        for k in 1..=64 {
            term = (&term * &a).map(|value| value / k as f64);
            result += &term;
            if Self::matrix_max_abs_entry(&term) < 1e-14 {
                break;
            }
        }

        let mut out = Vec::with_capacity(n * n);
        for row in 0..n {
            for col in 0..n {
                out.push(Self::from_complex64(result[(row, col)]));
            }
        }

        Matrix::new(n, n, out)
    }

    fn matrix_hermitian(matrix: &Matrix) -> Matrix {
        let mut out = vec![Complex { re: 0.0, im: 0.0 }; matrix.data.len()];
        for row in 0..matrix.rows {
            for col in 0..matrix.cols {
                let value = matrix.data[row * matrix.cols + col];
                out[col * matrix.rows + row] = Complex {
                    re: value.re,
                    im: -value.im,
                };
            }
        }
        Matrix {
            rows: matrix.cols,
            cols: matrix.rows,
            data: out,
        }
    }

    fn matrix_mat_pow(matrix: &Matrix, exponent: i64) -> Result<Matrix, CalcError> {
        Self::require_square(matrix, "MatPow")?;
        let n = matrix.rows;
        if exponent == 0 {
            return Ok(Self::matrix_identity(n));
        }

        let mut base = if exponent < 0 {
            Self::matrix_inverse(matrix)?
        } else {
            matrix.clone()
        };
        let mut exp = exponent.unsigned_abs();
        let mut result = Self::matrix_identity(n);

        while exp > 0 {
            if (exp & 1) == 1 {
                result = Self::matrix_mul(&result, &base)?;
            }
            exp >>= 1;
            if exp > 0 {
                base = Self::matrix_mul(&base, &base)?;
            }
        }

        Ok(result)
    }

    fn dmatrix_to_matrix(matrix: &DMatrix<Complex64>) -> Matrix {
        let rows = matrix.nrows();
        let cols = matrix.ncols();
        let mut out = Vec::with_capacity(rows * cols);
        for row in 0..rows {
            for col in 0..cols {
                out.push(Self::from_complex64(matrix[(row, col)]));
            }
        }
        Matrix {
            rows,
            cols,
            data: out,
        }
    }

    fn matrix_qr(matrix: &Matrix) -> Result<(Matrix, Matrix), CalcError> {
        let a = Self::matrix_to_dmatrix(matrix);
        let m = a.nrows();
        let n = a.ncols();
        let mut q = DMatrix::<Complex64>::zeros(m, n);
        let mut r = DMatrix::<Complex64>::zeros(n, n);
        let eps = 1e-12;

        for j in 0..n {
            let mut v = a.column(j).into_owned();
            for i in 0..j {
                let q_i = q.column(i);
                let rij = q_i.dotc(&v);
                r[(i, j)] = rij;
                v -= q_i * rij;
            }

            let norm = v.norm();
            if norm <= eps {
                return Err(CalcError::SingularMatrix(
                    "QR failed: matrix columns are linearly dependent".to_string(),
                ));
            }

            r[(j, j)] = Complex64::new(norm, 0.0);
            let qj = v.map(|value| value / norm);
            q.set_column(j, &qj);
        }

        Ok((Self::dmatrix_to_matrix(&q), Self::dmatrix_to_matrix(&r)))
    }

    fn matrix_svd(matrix: &Matrix) -> Result<(Matrix, Matrix, Matrix), CalcError> {
        let a = Self::matrix_to_dmatrix(matrix);
        let m = a.nrows();
        let n = a.ncols();
        let k = m.min(n);

        let svd = a.svd(true, true);
        let u = svd
            .u
            .ok_or_else(|| CalcError::SingularMatrix("SVD failed to produce U".to_string()))?;
        let vt = svd
            .v_t
            .ok_or_else(|| CalcError::SingularMatrix("SVD failed to produce Vt".to_string()))?;

        let mut s = DMatrix::<Complex64>::zeros(m, n);
        for i in 0..k {
            s[(i, i)] = Complex64::new(svd.singular_values[i], 0.0);
        }

        Ok((
            Self::dmatrix_to_matrix(&u),
            Self::dmatrix_to_matrix(&s),
            Self::dmatrix_to_matrix(&vt),
        ))
    }

    fn matrix_evd(matrix: &Matrix) -> Result<(Matrix, Matrix, Option<String>), CalcError> {
        Self::require_square(matrix, "EVD")?;

        let a = Self::matrix_to_dmatrix(matrix);
        let schur = Schur::new(a.clone());
        let (q, t) = schur.unpack();

        let mut off_diag_max = 0.0_f64;
        for row in 0..t.nrows() {
            for col in 0..t.ncols() {
                if row != col {
                    off_diag_max = off_diag_max.max(t[(row, col)].norm());
                }
            }
        }

        let warning = if off_diag_max > 1e-8 {
            Some(
                "EVD warning: exact diagonalization unavailable; returned Schur form (Q, T) with A = Q*T*Q^-1."
                    .to_string(),
            )
        } else {
            None
        };

        Ok((
            Self::dmatrix_to_matrix(&q),
            Self::dmatrix_to_matrix(&t),
            warning,
        ))
    }

    fn matrix_lu(matrix: &Matrix) -> Result<(Matrix, Matrix, Matrix), CalcError> {
        Self::require_square(matrix, "LU")?;

        let n = matrix.rows;
        let mut u = Self::matrix_to_dmatrix(matrix);
        let mut l = DMatrix::<Complex64>::identity(n, n);
        let mut p = DMatrix::<Complex64>::identity(n, n);
        let eps = 1e-12;

        for k in 0..n {
            let mut pivot_row = k;
            let mut pivot_abs = u[(k, k)].norm();
            for row in (k + 1)..n {
                let candidate = u[(row, k)].norm();
                if candidate > pivot_abs {
                    pivot_abs = candidate;
                    pivot_row = row;
                }
            }

            if pivot_abs <= eps {
                return Err(CalcError::SingularMatrix(
                    "LU failed: matrix is singular".to_string(),
                ));
            }

            if pivot_row != k {
                u.swap_rows(k, pivot_row);
                p.swap_rows(k, pivot_row);
                for col in 0..k {
                    let tmp = l[(k, col)];
                    l[(k, col)] = l[(pivot_row, col)];
                    l[(pivot_row, col)] = tmp;
                }
            }

            let pivot = u[(k, k)];
            for row in (k + 1)..n {
                let factor = u[(row, k)] / pivot;
                l[(row, k)] = factor;
                u[(row, k)] = Complex64::new(0.0, 0.0);
                for col in (k + 1)..n {
                    let upper = u[(k, col)];
                    u[(row, col)] -= factor * upper;
                }
            }
        }

        Ok((
            Self::dmatrix_to_matrix(&p),
            Self::dmatrix_to_matrix(&l),
            Self::dmatrix_to_matrix(&u),
        ))
    }

    fn matrix_real_vector(matrix: &Matrix) -> Result<Vec<f64>, CalcError> {
        if matrix.rows != 1 && matrix.cols != 1 {
            return Err(CalcError::TypeMismatch(format!(
                "expected vector shape Nx1 or 1xN, got {}x{}",
                matrix.rows, matrix.cols
            )));
        }

        let mut out = Vec::with_capacity(matrix.data.len());
        for value in &matrix.data {
            if value.im.abs() > 1e-12 {
                return Err(CalcError::TypeMismatch(
                    "statistics operations currently require real-valued vectors".to_string(),
                ));
            }
            out.push(value.re);
        }

        if out.is_empty() {
            return Err(CalcError::InvalidInput(
                "statistics operations require at least one value".to_string(),
            ));
        }

        Ok(out)
    }

    fn matrix_mean(matrix: &Matrix) -> Result<f64, CalcError> {
        let values = Self::matrix_real_vector(matrix)?;
        let sum = values.iter().sum::<f64>();
        Ok(sum / values.len() as f64)
    }

    fn matrix_mode(matrix: &Matrix) -> Result<f64, CalcError> {
        let values = Self::matrix_real_vector(matrix)?;
        let mut counts = std::collections::HashMap::<i64, (usize, f64)>::new();
        for value in values {
            let key = (value * 1e12).round() as i64;
            let entry = counts.entry(key).or_insert((0, value));
            entry.0 += 1;
        }

        let (_, mode) = counts
            .into_values()
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)))
            .ok_or_else(|| {
                CalcError::InvalidInput("mode requires at least one value".to_string())
            })?;

        Ok(mode)
    }

    fn matrix_variance(matrix: &Matrix) -> Result<f64, CalcError> {
        let values = Self::matrix_real_vector(matrix)?;
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let var = values
            .iter()
            .map(|value| {
                let d = *value - mean;
                d * d
            })
            .sum::<f64>()
            / values.len() as f64;
        Ok(var)
    }

    fn matrix_std_dev(matrix: &Matrix) -> Result<f64, CalcError> {
        Ok(Self::matrix_variance(matrix)?.sqrt())
    }

    fn matrix_max(matrix: &Matrix) -> Result<f64, CalcError> {
        let values = Self::matrix_real_vector(matrix)?;
        values
            .into_iter()
            .max_by(|a, b| a.total_cmp(b))
            .ok_or_else(|| CalcError::InvalidInput("max requires at least one value".to_string()))
    }

    fn matrix_min(matrix: &Matrix) -> Result<f64, CalcError> {
        let values = Self::matrix_real_vector(matrix)?;
        values
            .into_iter()
            .min_by(|a, b| a.total_cmp(b))
            .ok_or_else(|| CalcError::InvalidInput("min requires at least one value".to_string()))
    }

    fn matrix_transpose(matrix: &Matrix) -> Matrix {
        let mut out = vec![Complex { re: 0.0, im: 0.0 }; matrix.data.len()];
        for row in 0..matrix.rows {
            for col in 0..matrix.cols {
                out[col * matrix.rows + row] = matrix.data[row * matrix.cols + col];
            }
        }
        Matrix {
            rows: matrix.cols,
            cols: matrix.rows,
            data: out,
        }
    }

    fn matrix_identity(size: usize) -> Matrix {
        let mut data = vec![Complex { re: 0.0, im: 0.0 }; size * size];
        for i in 0..size {
            data[i * size + i] = Complex { re: 1.0, im: 0.0 };
        }
        Matrix {
            rows: size,
            cols: size,
            data,
        }
    }

    fn matrix_determinant(matrix: &Matrix) -> Result<Complex, CalcError> {
        Self::require_square(matrix, "determinant")?;
        let n = matrix.rows;
        let mut data = matrix
            .data
            .iter()
            .map(|v| Self::to_complex64(*v))
            .collect::<Vec<_>>();
        let mut sign = 1.0;
        let mut det = Complex64::new(1.0, 0.0);
        let eps = 1e-12;

        for i in 0..n {
            let mut pivot_row = i;
            let mut pivot_abs = data[i * n + i].norm();
            for r in (i + 1)..n {
                let candidate = data[r * n + i].norm();
                if candidate > pivot_abs {
                    pivot_abs = candidate;
                    pivot_row = r;
                }
            }

            if pivot_abs < eps {
                return Ok(Complex { re: 0.0, im: 0.0 });
            }

            if pivot_row != i {
                for c in 0..n {
                    data.swap(i * n + c, pivot_row * n + c);
                }
                sign *= -1.0;
            }

            let pivot = data[i * n + i];
            det *= pivot;

            for r in (i + 1)..n {
                let factor = data[r * n + i] / pivot;
                data[r * n + i] = Complex64::new(0.0, 0.0);
                for c in (i + 1)..n {
                    let upper = data[i * n + c];
                    data[r * n + c] -= factor * upper;
                }
            }
        }

        Ok(Self::from_complex64(det * sign))
    }

    fn matrix_inverse(matrix: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_square(matrix, "inverse")?;
        let n = matrix.rows;
        let mut a = matrix
            .data
            .iter()
            .map(|v| Self::to_complex64(*v))
            .collect::<Vec<_>>();
        let mut inv = Self::matrix_identity(n)
            .data
            .iter()
            .map(|v| Self::to_complex64(*v))
            .collect::<Vec<_>>();
        let eps = 1e-12;

        for i in 0..n {
            let mut pivot_row = i;
            let mut pivot_abs = a[i * n + i].norm();
            for r in (i + 1)..n {
                let candidate = a[r * n + i].norm();
                if candidate > pivot_abs {
                    pivot_abs = candidate;
                    pivot_row = r;
                }
            }

            if pivot_abs < eps {
                return Err(CalcError::SingularMatrix(
                    "inverse is undefined for singular matrices".to_string(),
                ));
            }

            if pivot_row != i {
                for c in 0..n {
                    a.swap(i * n + c, pivot_row * n + c);
                    inv.swap(i * n + c, pivot_row * n + c);
                }
            }

            let pivot = a[i * n + i];
            for c in 0..n {
                a[i * n + c] /= pivot;
                inv[i * n + c] /= pivot;
            }

            for r in 0..n {
                if r == i {
                    continue;
                }
                let factor = a[r * n + i];
                if factor.norm() < eps {
                    continue;
                }
                for c in 0..n {
                    let a_upper = a[i * n + c];
                    let inv_upper = inv[i * n + c];
                    a[r * n + c] -= factor * a_upper;
                    inv[r * n + c] -= factor * inv_upper;
                }
            }
        }
        Matrix::new(n, n, inv.into_iter().map(Self::from_complex64).collect())
    }

    fn matrix_solve(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_square(a, "solve_ax_b left operand")?;
        if a.rows != b.rows {
            return Err(CalcError::DimensionMismatch {
                expected: a.rows,
                actual: b.rows,
            });
        }

        let n = a.rows;
        let rhs_cols = b.cols;
        let mut a_data = a
            .data
            .iter()
            .map(|v| Self::to_complex64(*v))
            .collect::<Vec<_>>();
        let mut b_data = b
            .data
            .iter()
            .map(|v| Self::to_complex64(*v))
            .collect::<Vec<_>>();
        let eps = 1e-12;

        for i in 0..n {
            let mut pivot_row = i;
            let mut pivot_abs = a_data[i * n + i].norm();
            for r in (i + 1)..n {
                let candidate = a_data[r * n + i].norm();
                if candidate > pivot_abs {
                    pivot_abs = candidate;
                    pivot_row = r;
                }
            }

            if pivot_abs < eps {
                return Err(CalcError::SingularMatrix(
                    "solve_ax_b failed: singular coefficient matrix".to_string(),
                ));
            }

            if pivot_row != i {
                for c in 0..n {
                    a_data.swap(i * n + c, pivot_row * n + c);
                }
                for c in 0..rhs_cols {
                    b_data.swap(i * rhs_cols + c, pivot_row * rhs_cols + c);
                }
            }

            let pivot = a_data[i * n + i];
            for r in (i + 1)..n {
                let factor = a_data[r * n + i] / pivot;
                a_data[r * n + i] = Complex64::new(0.0, 0.0);
                for c in (i + 1)..n {
                    let upper = a_data[i * n + c];
                    a_data[r * n + c] -= factor * upper;
                }
                for c in 0..rhs_cols {
                    let rhs_upper = b_data[i * rhs_cols + c];
                    b_data[r * rhs_cols + c] -= factor * rhs_upper;
                }
            }
        }

        let mut x_data = vec![Complex64::new(0.0, 0.0); n * rhs_cols];
        for rhs_col in 0..rhs_cols {
            for i in (0..n).rev() {
                let mut sum = b_data[i * rhs_cols + rhs_col];
                for j in (i + 1)..n {
                    sum -= a_data[i * n + j] * x_data[j * rhs_cols + rhs_col];
                }
                let pivot = a_data[i * n + i];
                if pivot.norm() < eps {
                    return Err(CalcError::SingularMatrix(
                        "solve_ax_b failed during back substitution".to_string(),
                    ));
                }
                x_data[i * rhs_cols + rhs_col] = sum / pivot;
            }
        }
        Matrix::new(
            n,
            rhs_cols,
            x_data.into_iter().map(Self::from_complex64).collect(),
        )
    }

    fn matrix_solve_lstsq(a: &Matrix, b: &Matrix) -> Result<(Matrix, Option<String>), CalcError> {
        if a.rows != b.rows {
            return Err(CalcError::DimensionMismatch {
                expected: a.rows,
                actual: b.rows,
            });
        }

        let a_dm = Self::matrix_to_dmatrix(a);
        let b_dm = Self::matrix_to_dmatrix(b);
        let svd = a_dm.clone().svd(true, true);
        let max_sigma = svd.singular_values.iter().copied().fold(0.0_f64, f64::max);
        let eps = 1e-12;
        let rank = if max_sigma <= eps {
            0
        } else {
            let threshold = max_sigma * eps;
            svd.singular_values
                .iter()
                .filter(|&&sigma| sigma > threshold)
                .count()
        };
        let full_rank = rank == a.rows.min(a.cols);

        let pinv = svd.pseudo_inverse(eps).map_err(|_| {
            CalcError::SingularMatrix(
                "solve_lstsq failed: could not compute pseudoinverse".to_string(),
            )
        })?;
        let x = pinv * b_dm;
        let residual = a_dm * x.clone() - Self::matrix_to_dmatrix(b);
        let residual_norm = residual.norm();

        let warning = if full_rank {
            Some(format!("LSTSQ residual norm: {:.6e}", residual_norm))
        } else {
            Some(format!(
                "LSTSQ warning: rank-deficient system (rank {rank}/{}); returned minimum-norm pseudoinverse solution. Residual norm: {:.6e}",
                a.rows.min(a.cols),
                residual_norm
            ))
        };

        Ok((Self::dmatrix_to_matrix(&x), warning))
    }

    fn require_same_shape(a: &Matrix, b: &Matrix, operation: &str) -> Result<(), CalcError> {
        if a.rows != b.rows || a.cols != b.cols {
            return Err(CalcError::TypeMismatch(format!(
                "{operation} requires equal matrix dimensions: left is {}x{}, right is {}x{}",
                a.rows, a.cols, b.rows, b.cols
            )));
        }
        Ok(())
    }

    fn require_square(matrix: &Matrix, operation: &str) -> Result<(), CalcError> {
        if matrix.rows != matrix.cols {
            return Err(CalcError::TypeMismatch(format!(
                "{operation} requires a square matrix but got {}x{}",
                matrix.rows, matrix.cols
            )));
        }
        Ok(())
    }
}


#[cfg(test)]
#[path = "tests/lib_tests.rs"]
mod tests;
