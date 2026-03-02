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
            (Value::Matrix(a), Value::Real(b)) => Ok(Value::Matrix(Self::matrix_scalar_mul(a, *b))),
            (Value::Real(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_scalar_mul(b, *a))),
            (Value::Real(a), Value::Real(b)) => Ok(Value::Real(a * b)),
            (Value::Matrix(_), _) | (_, Value::Matrix(_)) => Err(CalcError::TypeMismatch(
                "* only supports matrix*matrix, matrix*real, or scalar/complex arithmetic"
                    .to_string(),
            )),
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
            Value::Matrix(matrix) => Ok(Value::Real(Self::matrix_determinant(matrix)?)),
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

    fn matrix_add(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_same_shape(a, b, "matrix add")?;
        let data = a
            .data
            .iter()
            .zip(&b.data)
            .map(|(lhs, rhs)| lhs + rhs)
            .collect::<Vec<_>>();
        Matrix::new(a.rows, a.cols, data)
    }

    fn matrix_sub(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_same_shape(a, b, "matrix sub")?;
        let data = a
            .data
            .iter()
            .zip(&b.data)
            .map(|(lhs, rhs)| lhs - rhs)
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

        let mut out = vec![0.0; a.rows * b.cols];
        for row in 0..a.rows {
            for col in 0..b.cols {
                let mut acc = 0.0;
                for k in 0..a.cols {
                    acc += a.data[row * a.cols + k] * b.data[k * b.cols + col];
                }
                out[row * b.cols + col] = acc;
            }
        }
        Matrix::new(a.rows, b.cols, out)
    }

    fn matrix_scalar_mul(matrix: &Matrix, scalar: f64) -> Matrix {
        let data = matrix.data.iter().map(|value| value * scalar).collect();
        Matrix {
            rows: matrix.rows,
            cols: matrix.cols,
            data,
        }
    }

    fn matrix_transpose(matrix: &Matrix) -> Matrix {
        let mut out = vec![0.0; matrix.data.len()];
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
        let mut data = vec![0.0; size * size];
        for i in 0..size {
            data[i * size + i] = 1.0;
        }
        Matrix {
            rows: size,
            cols: size,
            data,
        }
    }

    fn matrix_determinant(matrix: &Matrix) -> Result<f64, CalcError> {
        Self::require_square(matrix, "determinant")?;
        let n = matrix.rows;
        let mut data = matrix.data.clone();
        let mut sign = 1.0;
        let mut det = 1.0;
        let eps = 1e-12;

        for i in 0..n {
            let mut pivot_row = i;
            let mut pivot_abs = data[i * n + i].abs();
            for r in (i + 1)..n {
                let candidate = data[r * n + i].abs();
                if candidate > pivot_abs {
                    pivot_abs = candidate;
                    pivot_row = r;
                }
            }

            if pivot_abs < eps {
                return Ok(0.0);
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
                data[r * n + i] = 0.0;
                for c in (i + 1)..n {
                    data[r * n + c] -= factor * data[i * n + c];
                }
            }
        }

        Ok(sign * det)
    }

    fn matrix_inverse(matrix: &Matrix) -> Result<Matrix, CalcError> {
        Self::require_square(matrix, "inverse")?;
        let n = matrix.rows;
        let mut a = matrix.data.clone();
        let mut inv = Self::matrix_identity(n).data;
        let eps = 1e-12;

        for i in 0..n {
            let mut pivot_row = i;
            let mut pivot_abs = a[i * n + i].abs();
            for r in (i + 1)..n {
                let candidate = a[r * n + i].abs();
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
                if factor.abs() < eps {
                    continue;
                }
                for c in 0..n {
                    a[r * n + c] -= factor * a[i * n + c];
                    inv[r * n + c] -= factor * inv[i * n + c];
                }
            }
        }

        Matrix::new(n, n, inv)
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
        let mut a_data = a.data.clone();
        let mut b_data = b.data.clone();
        let eps = 1e-12;

        for i in 0..n {
            let mut pivot_row = i;
            let mut pivot_abs = a_data[i * n + i].abs();
            for r in (i + 1)..n {
                let candidate = a_data[r * n + i].abs();
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
                a_data[r * n + i] = 0.0;
                for c in (i + 1)..n {
                    a_data[r * n + c] -= factor * a_data[i * n + c];
                }
                for c in 0..rhs_cols {
                    b_data[r * rhs_cols + c] -= factor * b_data[i * rhs_cols + c];
                }
            }
        }

        let mut x_data = vec![0.0; n * rhs_cols];
        for rhs_col in 0..rhs_cols {
            for i in (0..n).rev() {
                let mut sum = b_data[i * rhs_cols + rhs_col];
                for j in (i + 1)..n {
                    sum -= a_data[i * n + j] * x_data[j * rhs_cols + rhs_col];
                }
                let pivot = a_data[i * n + i];
                if pivot.abs() < eps {
                    return Err(CalcError::SingularMatrix(
                        "solve_ax_b failed during back substitution".to_string(),
                    ));
                }
                x_data[i * rhs_cols + rhs_col] = sum / pivot;
            }
        }

        Matrix::new(n, rhs_cols, x_data)
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
        Matrix::new(rows, cols, data.to_vec()).expect("valid matrix")
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
            assert_real_close(*a, *e, eps);
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
    fn matrix_and_complex_multiplication_errors_without_mutation() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(1, 1, &[3.0])));
        calc.push_value(Value::Complex(Complex { re: 2.0, im: 1.0 }));
        let before = calc.state().stack.clone();

        let result = calc.mul();

        assert_eq!(
            result,
            Err(CalcError::TypeMismatch(
                "* only supports matrix*matrix, matrix*real, or scalar/complex arithmetic"
                    .to_string()
            ))
        );
        assert_eq!(calc.state().stack, before);
    }

    #[test]
    fn determinant_of_square_matrix() {
        let mut calc = Calculator::new();
        calc.push_value(Value::Matrix(matrix(2, 2, &[1.0, 2.0, 3.0, 4.0])));

        let result = calc.determinant();

        assert_eq!(result, Ok(()));
        match calc.state().stack.last() {
            Some(Value::Real(v)) => assert_real_close(*v, -2.0, 1e-12),
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
}
