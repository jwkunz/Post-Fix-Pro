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
            return Err(CalcError::InvalidInput(
                "entry buffer is empty".to_string(),
            ));
        }

        let value = self
            .state
            .entry_buffer
            .parse::<f64>()
            .map_err(|_| CalcError::InvalidInput("entry buffer is not a valid number".to_string()))?;

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

    pub fn add(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_add(a, b)?)),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a + b)),
            (Value::Matrix(_), _) | (_, Value::Matrix(_)) => Err(CalcError::TypeMismatch(
                "+ only supports matrix+matrix or scalar/complex arithmetic".to_string(),
            )),
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
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a - b)),
            (Value::Matrix(_), _) | (_, Value::Matrix(_)) => Err(CalcError::TypeMismatch(
                "- only supports matrix-matrix or scalar/complex arithmetic".to_string(),
            )),
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
            (Value::Real(_,), Value::Real(b)) if *b == 0.0 => Err(CalcError::DivideByZero),
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

    pub fn sqrt(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) if *v < 0.0 => Err(CalcError::DomainError(
                "sqrt is undefined for negative real values".to_string(),
            )),
            Value::Real(v) => Ok(Value::Real(v.sqrt())),
            Value::Complex(c) => Ok(Value::Complex(Self::complex_sqrt(*c))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "sqrt does not support matrix values".to_string(),
            )),
        })
    }

    pub fn exp(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.exp())),
            Value::Complex(c) => Ok(Value::Complex(Self::complex_exp(*c))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "exp does not support matrix values".to_string(),
            )),
        })
    }

    pub fn ln(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) if *v <= 0.0 => Err(CalcError::DomainError(
                "ln is undefined for non-positive real values".to_string(),
            )),
            Value::Real(v) => Ok(Value::Real(v.ln())),
            Value::Complex(c) => Ok(Value::Complex(Self::complex_ln(*c))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "ln does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "sin does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "cos does not support matrix values".to_string(),
            )),
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
                let denom_norm =
                    denominator.re * denominator.re + denominator.im * denominator.im;
                if denom_norm == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Complex {
                    re: (numerator.re * denominator.re + numerator.im * denominator.im) / denom_norm,
                    im: (numerator.im * denominator.re - numerator.re * denominator.im) / denom_norm,
                }))
            }
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "tan does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "asin does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "acos does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "atan does not support matrix values".to_string(),
            )),
        })
    }

    pub fn sinh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.sinh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).sinh(),
            ))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "sinh does not support matrix values".to_string(),
            )),
        })
    }

    pub fn cosh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.cosh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).cosh(),
            ))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "cosh does not support matrix values".to_string(),
            )),
        })
    }

    pub fn tanh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.tanh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).tanh(),
            ))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "tanh does not support matrix values".to_string(),
            )),
        })
    }

    pub fn asinh(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.asinh())),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Self::to_complex64(*c).asinh(),
            ))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "asinh does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "acosh does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "atanh does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "log10 does not support matrix values".to_string(),
            )),
        })
    }

    pub fn gamma(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(Self::real_gamma(*v))),
            Value::Complex(_) => Err(CalcError::TypeMismatch(
                "gamma currently supports real values only".to_string(),
            )),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "gamma does not support matrix values".to_string(),
            )),
        })
    }

    pub fn erf(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(Self::real_erf(*v))),
            Value::Complex(_) => Err(CalcError::TypeMismatch(
                "erf currently supports real values only".to_string(),
            )),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "erf does not support matrix values".to_string(),
            )),
        })
    }

    pub fn pow(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "inv does not support matrix values".to_string(),
            )),
        })
    }

    pub fn square(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v * v)),
            Value::Complex(c) => Ok(Value::Complex(Complex {
                re: c.re * c.re - c.im * c.im,
                im: 2.0 * c.re * c.im,
            })),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "square does not support matrix values".to_string(),
            )),
        })
    }

    pub fn root(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(x), Value::Real(y)) => {
                if *y == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(x.powf(1.0 / y)))
            }
            _ => {
                let x = Self::as_complex(left, "root")?;
                let y = Self::as_complex(right, "root")?;
                let out = Self::to_complex64(x).powc(Complex64::new(1.0, 0.0) / Self::to_complex64(y));
                Ok(Value::Complex(Self::from_complex64(out)))
            }
        })
    }

    pub fn exp10(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(10.0_f64.powf(*v))),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Complex64::new(10.0, 0.0).powc(Self::to_complex64(*c)),
            ))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "10^x does not support matrix values".to_string(),
            )),
        })
    }

    pub fn exp2(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(2.0_f64.powf(*v))),
            Value::Complex(c) => Ok(Value::Complex(Self::from_complex64(
                Complex64::new(2.0, 0.0).powc(Self::to_complex64(*c)),
            ))),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "2^x does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "log2 does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "signum does not support matrix values".to_string(),
            )),
        })
    }

    pub fn abs(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.abs())),
            Value::Complex(c) => Ok(Value::Real((c.re * c.re + c.im * c.im).sqrt())),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "abs does not support matrix values".to_string(),
            )),
        })
    }

    pub fn abs_sq(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v * v)),
            Value::Complex(c) => Ok(Value::Real(c.re * c.re + c.im * c.im)),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "abs^2 does not support matrix values".to_string(),
            )),
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
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "arg does not support matrix values".to_string(),
            )),
        })
    }

    pub fn conjugate(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Complex(c) => Ok(Value::Complex(Complex {
                re: c.re,
                im: -c.im,
            })),
            Value::Real(v) => Ok(Value::Real(*v)),
            Value::Matrix(_) => Err(CalcError::TypeMismatch(
                "conjugate does not support matrix values".to_string(),
            )),
        })
    }

    pub fn atan2(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_binary_op(|left, right| match (left, right) {
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
        })
    }

    pub fn to_rad(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.to_radians())),
            _ => Err(CalcError::TypeMismatch(
                "to_rad currently supports real values only".to_string(),
            )),
        })
    }

    pub fn to_deg(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.to_degrees())),
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
            _ => Err(CalcError::TypeMismatch(
                "rnd currently supports real values only".to_string(),
            )),
        })
    }

    pub fn floor_value(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.floor())),
            _ => Err(CalcError::TypeMismatch(
                "floor currently supports real values only".to_string(),
            )),
        })
    }

    pub fn ceil_value(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.ceil())),
            _ => Err(CalcError::TypeMismatch(
                "ceil currently supports real values only".to_string(),
            )),
        })
    }

    pub fn dec_part(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v - v.trunc())),
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
        self.state.stack.push(Value::Matrix(Self::matrix_identity(size)));
        Ok(())
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
        let value = self.state.stack.get(len - 1).expect("prechecked stack length");
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
        let left = self.state.stack.get(len - 2).expect("prechecked stack length");
        let right = self.state.stack.get(len - 1).expect("prechecked stack length");
        let result = op(left, right)?;
        self.state.stack.truncate(len - 2);
        self.state.stack.push(result);
        Ok(())
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
            return Err(CalcError::InvalidInput(format!("{label} must be an integer")));
        }
        if value < i64::MIN as f64 || value > i64::MAX as f64 {
            return Err(CalcError::InvalidInput(format!("{label} is out of range")));
        }
        Ok(value as i64)
    }

    fn as_non_negative_integer(value: f64, label: &str) -> Result<u64, CalcError> {
        let int = Self::as_integer(value, label)?;
        if int < 0 {
            return Err(CalcError::DomainError(format!("{label} must be non-negative")));
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
        Matrix::new(
            n,
            n,
            inv.into_iter().map(Self::from_complex64).collect(),
        )
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
mod tests {
    use super::{AngleMode, CalcError, Calculator, Complex, Matrix, Value};

    fn matrix(rows: usize, cols: usize, data: &[f64]) -> Matrix {
        let complex_data = data
            .iter()
            .map(|value| Complex {
                re: *value,
                im: 0.0,
            })
            .collect::<Vec<_>>();
        Matrix::new(rows, cols, complex_data).expect("valid matrix")
    }

    fn assert_real_close(actual: f64, expected: f64, eps: f64) {
        assert!(
            (actual - expected).abs() <= eps,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_matrix_close(actual: &Matrix, expected: &Matrix, eps: f64) {
        assert_eq!(actual.rows, expected.rows);
        assert_eq!(actual.cols, expected.cols);
        for (a, e) in actual.data.iter().zip(&expected.data) {
            assert_real_close(a.re, e.re, eps);
            assert_real_close(a.im, e.im, eps);
        }
    }

    #[test]
    fn enter_pushes_real_and_clears_entry() {
        let mut calc = Calculator::new();
        calc.entry_set("12.5");

        let result = calc.enter();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(12.5)]);
        assert_eq!(calc.state().entry_buffer, "");
    }

    #[test]
    fn enter_with_invalid_input_preserves_state() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(9.0));
        calc.entry_set("abc");

        let result = calc.enter();

        assert_eq!(
            result,
            Err(CalcError::InvalidInput(
                "entry buffer is not a valid number".to_string()
            ))
        );
        assert_eq!(calc.state().stack, vec![Value::Real(9.0)]);
        assert_eq!(calc.state().entry_buffer, "abc");
    }

    #[test]
    fn drop_returns_top_value() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(3.0));
        calc.push_value(Value::Real(7.0));

        let dropped = calc.drop();

        assert_eq!(dropped, Ok(Value::Real(7.0)));
        assert_eq!(calc.state().stack, vec![Value::Real(3.0)]);
    }

    #[test]
    fn dup_copies_top_value() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(2.0));

        let result = calc.dup();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(2.0), Value::Real(2.0)]);
    }

    #[test]
    fn swap_exchanges_top_two_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));

        let result = calc.swap();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(2.0), Value::Real(1.0)]);
    }

    #[test]
    fn rot_rotates_top_three_values_left() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(3.0));

        let result = calc.rot();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Real(2.0), Value::Real(3.0), Value::Real(1.0)]
        );
    }

    #[test]
    fn stack_underflow_errors_do_not_modify_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(10.0));

        let dup_before = calc.state().stack.clone();
        let swap_result = calc.swap();

        assert_eq!(
            swap_result,
            Err(CalcError::StackUnderflow {
                needed: 2,
                available: 1
            })
        );
        assert_eq!(calc.state().stack, dup_before);
    }

    #[test]
    fn add_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(10.0));
        calc.push_value(Value::Real(5.0));

        let result = calc.add();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(15.0)]);
    }

    #[test]
    fn add_mixed_values_promotes_to_complex() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Complex(Complex { re: 3.0, im: 4.0 }));

        let result = calc.add();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Complex(Complex { re: 5.0, im: 4.0 })]
        );
    }

    #[test]
    fn div_by_zero_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(12.0));
        calc.push_value(Value::Real(0.0));
        let before = calc.state().stack.clone();

        let result = calc.div();

        assert_eq!(result, Err(CalcError::DivideByZero));
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn sqrt_negative_real_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(-9.0));
        let before = calc.state().stack.clone();

        let result = calc.sqrt();

        assert_eq!(
            result,
            Err(CalcError::DomainError(
                "sqrt is undefined for negative real values".to_string()
            ))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn sin_respects_degree_mode_for_real_values() {
        let mut calc = Calculator::new();
        calc.set_angle_mode(AngleMode::Deg);
        calc.push_value(Value::Real(90.0));

        let result = calc.sin();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert!((v - 1.0).abs() < 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn ln_non_positive_real_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(0.0));
        let before = calc.state().stack.clone();

        let result = calc.ln();

        assert_eq!(
            result,
            Err(CalcError::DomainError(
                "ln is undefined for non-positive real values".to_string()
            ))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn add_two_matrices() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));
        calc.push_value(Value::Matrix(matrix(2, 2, &[5.0, 6.0, 7.0, 8.0])));

        let result = calc.add();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(2, 2, &[6.0, 8.0, 10.0, 12.0]))]
        );
    }

    #[test]
    fn matrix_add_shape_mismatch_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));
        calc.push_value(Value::Matrix(matrix(1, 3, &[5.0, 6.0, 7.0])));
        let before = calc.state().stack.clone();

        let result = calc.add();

        assert!(
            matches!(result, Err(CalcError::TypeMismatch(message)) if message.contains("equal matrix dimensions"))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn mul_two_matrices() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
        calc.push_value(Value::Matrix(matrix(3, 2, &[7.0, 8.0, 9.0, 10.0, 11.0, 12.0])));

        let result = calc.mul();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(2, 2, &[58.0, 64.0, 139.0, 154.0]))]
        );
    }

    #[test]
    fn matrix_times_scalar() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, -2.0, 3.0, -4.0])));
        calc.push_value(Value::Real(2.5));

        let result = calc.mul();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(2, 2, &[2.5, -5.0, 7.5, -10.0]))]
        );
    }

    #[test]
    fn transpose_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));

        let result = calc.transpose();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(3, 2, &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0]))]
        );
    }

    #[test]
    fn push_identity_matrix() {
        let mut calc = Calculator::new();

        let result = calc.push_identity(3);

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(matrix(
                3,
                3,
                &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
            ))]
        );
    }

    #[test]
    fn matrix_and_complex_multiplication_scales_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(1, 1, &[3.0])));
        calc.push_value(Value::Complex(Complex { re: 2.0, im: 1.0 }));

        let result = calc.mul();

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Matrix(Matrix::new(
                1,
                1,
                vec![Complex { re: 6.0, im: 3.0 }]
            )
            .expect("valid matrix"))]
        );
    }

    #[test]
    fn determinant_of_square_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));

        let result = calc.determinant();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Complex(v)) => {
                assert_real_close(v.re, -2.0, 1e-12);
                assert_real_close(v.im, 0.0, 1e-12);
            }
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn inverse_of_square_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[4.0, 7.0, 2.0, 6.0])));

        let result = calc.inverse();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Matrix(actual)) => {
                let expected = matrix(2, 2, &[0.6, -0.7, -0.2, 0.4]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn solve_ax_b_with_vector_rhs() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[3.0, 2.0, 1.0, 2.0])));
        calc.push_value(Value::Matrix(matrix(2, 1, &[5.0, 5.0])));

        let result = calc.solve_ax_b();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Matrix(actual)) => {
                let expected = matrix(2, 1, &[0.0, 2.5]);
                assert_matrix_close(actual, &expected, 1e-12);
            }
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn inverse_of_singular_matrix_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 2.0, 4.0])));
        let before = calc.state().stack.clone();

        let result = calc.inverse();

        assert_eq!(
            result,
            Err(CalcError::SingularMatrix(
                "inverse is undefined for singular matrices".to_string()
            ))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn solve_ax_b_dimension_mismatch_preserves_stack() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 0.0, 0.0, 1.0])));
        calc.push_value(Value::Matrix(matrix(3, 1, &[1.0, 2.0, 3.0])));
        let before = calc.state().stack.clone();

        let result = calc.solve_ax_b();

        assert_eq!(
            result,
            Err(CalcError::DimensionMismatch {
                expected: 2,
                actual: 3
            })
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn pow_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(3.0));

        let result = calc.pow();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(8.0)]);
    }

    #[test]
    fn percent_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(200.0));
        calc.push_value(Value::Real(15.0));

        let result = calc.percent();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(30.0)]);
    }

    #[test]
    fn asin_in_degree_mode() {
        let mut calc = Calculator::new();
        calc.set_angle_mode(AngleMode::Deg);
        calc.push_value(Value::Real(0.5));

        let result = calc.asin();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 30.0, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn hyperbolic_functions_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        assert_eq!(calc.sinh(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 1.175_201_193_643_801_4, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }

        calc.push_value(Value::Real(1.0));
        assert_eq!(calc.cosh(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 1.543_080_634_815_243_7, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn log10_real_value() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1000.0));

        let result = calc.log10();

        assert_eq!(result, Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(3.0)]);
    }

    #[test]
    fn gamma_and_erf_real_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(5.0));
        assert_eq!(calc.gamma(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 24.0, 1e-9),
            other => panic!("unexpected stack value: {other:?}"),
        }

        calc.push_value(Value::Real(1.0));
        assert_eq!(calc.erf(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 0.842_700_79, 1e-6),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn push_constants() {
        let mut calc = Calculator::new();
        calc.push_pi();
        calc.push_e();

        assert_eq!(calc.state().stack.len(), 2);
        match &calc.state().stack[0] {
            Value::Real(v) => assert_real_close(*v, std::f64::consts::PI, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
        match &calc.state().stack[1] {
            Value::Real(v) => assert_real_close(*v, std::f64::consts::E, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }
    }

    #[test]
    fn memory_store_recall_and_clear() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(42.0));

        assert_eq!(calc.memory_store(0), Ok(()));
        assert_eq!(calc.clear_all(), ());
        assert_eq!(calc.memory_recall(0), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(42.0)]);
        assert_eq!(calc.memory_clear(0), Ok(()));
        assert_eq!(calc.memory_recall(0), Err(CalcError::EmptyRegister(0)));
    }

    #[test]
    fn memory_invalid_register_error() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        assert_eq!(calc.memory_store(26), Err(CalcError::InvalidRegister(26)));
        assert_eq!(calc.memory_recall(99), Err(CalcError::InvalidRegister(99)));
        assert_eq!(calc.memory_clear(999), Err(CalcError::InvalidRegister(999)));
    }

    #[test]
    fn roll_rotates_top_n_values() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(1.0));
        calc.push_value(Value::Real(2.0));
        calc.push_value(Value::Real(3.0));
        calc.push_value(Value::Real(4.0));

        let result = calc.roll(4);

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![
                Value::Real(2.0),
                Value::Real(3.0),
                Value::Real(4.0),
                Value::Real(1.0)
            ]
        );
    }

    #[test]
    fn pick_duplicates_nth_from_top() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(10.0));
        calc.push_value(Value::Real(20.0));
        calc.push_value(Value::Real(30.0));

        let result = calc.pick(2);

        assert_eq!(result, Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![
                Value::Real(10.0),
                Value::Real(20.0),
                Value::Real(30.0),
                Value::Real(20.0)
            ]
        );
    }

    #[test]
    fn complex_abs_arg_and_conjugate() {
        let mut calc = Calculator::new();
        calc.set_angle_mode(AngleMode::Deg);
        calc.push_value(Value::Complex(Complex { re: 3.0, im: 4.0 }));
        assert_eq!(calc.abs(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(5.0)]);

        calc.clear_all();
        calc.push_value(Value::Complex(Complex { re: 0.0, im: 1.0 }));
        assert_eq!(calc.arg(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 90.0, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Complex(Complex { re: -2.0, im: 7.0 }));
        assert_eq!(calc.conjugate(), Ok(()));
        assert_eq!(
            calc.state().stack,
            vec![Value::Complex(Complex { re: -2.0, im: -7.0 })]
        );
    }

    #[test]
    fn root_and_log2_exp2() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(27.0));
        calc.push_value(Value::Real(3.0));
        assert_eq!(calc.root(), Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, 3.0, 1e-12),
            other => panic!("unexpected stack value: {other:?}"),
        }

        calc.clear_all();
        calc.push_value(Value::Real(8.0));
        assert_eq!(calc.log2(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(3.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(5.0));
        assert_eq!(calc.exp2(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(32.0)]);
    }

    #[test]
    fn factorial_combinations_and_integer_ops() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Real(5.0));
        assert_eq!(calc.factorial(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(120.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(5.0));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.ncr(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(10.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(5.0));
        calc.push_value(Value::Real(2.0));
        assert_eq!(calc.npr(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(20.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(42.0));
        calc.push_value(Value::Real(30.0));
        assert_eq!(calc.gcd(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(6.0)]);

        calc.clear_all();
        calc.push_value(Value::Real(12.0));
        calc.push_value(Value::Real(18.0));
        assert_eq!(calc.lcm(), Ok(()));
        assert_eq!(calc.state().stack, vec![Value::Real(36.0)]);
    }
}
