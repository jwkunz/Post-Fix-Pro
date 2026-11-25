//! Calculator engine host and internal helpers.
//!
//! Public operation methods are split into panel-aligned modules under
//! `src/calculator_ops/`, while this file keeps shared helpers and matrix math.

use nalgebra::linalg::Schur;
use nalgebra::{DMatrix, DVector, Vector3};
use num_complex::Complex64;

use crate::types::{
    AngleMode, CalcError, CalcState, Complex, ComplexTransformMode, DisplayMode, Matrix, Value,
};

#[path = "calculator_ops/core_panel.rs"]
mod core_panel;
#[path = "calculator_ops/stack_panel.rs"]
mod stack_panel;
#[path = "calculator_ops/scalar_panel.rs"]
mod scalar_panel;
#[path = "calculator_ops/trigonometry_panel.rs"]
mod trigonometry_panel;
#[path = "calculator_ops/complex_panel.rs"]
mod complex_panel;
#[path = "calculator_ops/number_theory_panel.rs"]
mod number_theory_panel;
#[path = "calculator_ops/rounding_panel.rs"]
mod rounding_panel;
#[path = "calculator_ops/memory_panel.rs"]
mod memory_panel;
#[path = "calculator_ops/matrix_panel.rs"]
mod matrix_panel;
#[path = "calculator_ops/statistics_panel.rs"]
mod statistics_panel;

/// Stateful RPN calculator engine.
#[derive(Debug, Clone, PartialEq)]
pub struct Calculator {
    state: CalcState,
    undo_state: Option<CalcState>,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    /// Constructs a new instance.
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
            undo_state: None,
        }
    }

    /// Returns an immutable view of the current calculator state.
    pub fn state(&self) -> &CalcState {
        &self.state
    }

    /// Restores the calculator to the snapshot captured before the last
    /// successful mutating API operation.
    pub fn undo(&mut self) -> Result<(), CalcError> {
        let previous = self
            .undo_state
            .take()
            .ok_or(CalcError::InvalidInput("nothing to undo".to_string()))?;
        self.state = previous;
        Ok(())
    }

    pub(crate) fn set_undo_state(&mut self, snapshot: CalcState) {
        self.undo_state = Some(snapshot);
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

    fn real_erfc(x: f64) -> f64 {
        1.0 - Self::real_erf(x)
    }

    fn real_bessel_j0(x: f64) -> f64 {
        let mut sum = 1.0;
        let mut term = 1.0;
        let half_sq = 0.25 * x * x;
        for k in 1..=40 {
            let kf = k as f64;
            term *= -half_sq / (kf * kf);
            sum += term;
            if term.abs() < 1e-15 {
                break;
            }
        }
        sum
    }

    fn real_modified_bessel_i0(x: f64) -> f64 {
        let mut sum = 1.0;
        let mut term = 1.0;
        let half_sq = 0.25 * x * x;
        for k in 1..=40 {
            let kf = k as f64;
            term *= half_sq / (kf * kf);
            sum += term;
            if term.abs() < 1e-15 {
                break;
            }
        }
        sum
    }

    fn real_sinc(x: f64) -> f64 {
        if x.abs() < 1e-12 {
            1.0
        } else {
            x.sin() / x
        }
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

    pub(crate) fn matrix_mul(a: &Matrix, b: &Matrix) -> Result<Matrix, CalcError> {
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

    fn matrix_variance_population(matrix: &Matrix) -> Result<f64, CalcError> {
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

    fn matrix_variance_sample(matrix: &Matrix) -> Result<f64, CalcError> {
        let values = Self::matrix_real_vector(matrix)?;
        if values.len() < 2 {
            return Err(CalcError::InvalidInput(
                "sample variance requires at least two values".to_string(),
            ));
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let var = values
            .iter()
            .map(|value| {
                let d = *value - mean;
                d * d
            })
            .sum::<f64>()
            / (values.len() as f64 - 1.0);
        Ok(var)
    }

    fn matrix_std_dev_population(matrix: &Matrix) -> Result<f64, CalcError> {
        Ok(Self::matrix_variance_population(matrix)?.sqrt())
    }

    fn matrix_std_dev_sample(matrix: &Matrix) -> Result<f64, CalcError> {
        Ok(Self::matrix_variance_sample(matrix)?.sqrt())
    }

    fn matrix_median(matrix: &Matrix) -> Result<f64, CalcError> {
        let mut values = Self::matrix_real_vector(matrix)?;
        values.sort_by(|a, b| a.total_cmp(b));
        let n = values.len();
        if n % 2 == 1 {
            Ok(values[n / 2])
        } else {
            Ok((values[n / 2 - 1] + values[n / 2]) / 2.0)
        }
    }

    fn quantile_sorted(values: &[f64], q: f64) -> f64 {
        if values.len() == 1 {
            return values[0];
        }
        let pos = q.clamp(0.0, 1.0) * (values.len() as f64 - 1.0);
        let lo = pos.floor() as usize;
        let hi = pos.ceil() as usize;
        if lo == hi {
            values[lo]
        } else {
            let t = pos - lo as f64;
            values[lo] * (1.0 - t) + values[hi] * t
        }
    }

    fn matrix_quartiles_summary(matrix: &Matrix) -> Result<Matrix, CalcError> {
        let mut values = Self::matrix_real_vector(matrix)?;
        values.sort_by(|a, b| a.total_cmp(b));
        let min = *values.first().expect("prechecked non-empty");
        let max = *values.last().expect("prechecked non-empty");
        let q1 = Self::quantile_sorted(&values, 0.25);
        let q2 = Self::quantile_sorted(&values, 0.5);
        let q3 = Self::quantile_sorted(&values, 0.75);
        Matrix::new(
            1,
            5,
            vec![
                Complex { re: min, im: 0.0 },
                Complex { re: q1, im: 0.0 },
                Complex { re: q2, im: 0.0 },
                Complex { re: q3, im: 0.0 },
                Complex { re: max, im: 0.0 },
            ],
        )
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

    pub(crate) fn matrix_inverse(matrix: &Matrix) -> Result<Matrix, CalcError> {
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
