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
    pub data: Vec<f64>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<f64>) -> Result<Self, CalcError> {
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
    DomainError(String),
    DivideByZero,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcState {
    pub stack: Vec<Value>,
    pub entry_buffer: String,
    pub angle_mode: AngleMode,
    pub display_mode: DisplayMode,
    pub precision: u8,
    pub memory: Vec<Option<Value>>,
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
            },
        }
    }

    pub fn state(&self) -> &CalcState {
        &self.state
    }

    pub fn set_angle_mode(&mut self, mode: AngleMode) {
        self.state.angle_mode = mode;
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

    pub fn add(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
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
}

#[cfg(test)]
mod tests {
    use super::{AngleMode, CalcError, Calculator, Complex, Value};

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
}
