//! Calculator operations for the scalar panel panel.

use super::*;

impl Calculator {
    /// Executes the `add` operation.
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

    /// Executes the `sub` operation.
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

    /// Executes the `mul` operation.
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

    /// Executes the `div` operation.
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

    /// Executes the `hadamard_mul` operation.
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

    /// Executes the `hadamard_div` operation.
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

    /// Executes the `neg` operation.
    pub fn neg(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(-v)),
            Value::Complex(c) => Ok(Value::Complex(Complex {
                re: -c.re,
                im: -c.im,
            })),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                Ok(Complex {
                    re: -entry.re,
                    im: -entry.im,
                })
            })?)),
        })
    }

    /// Executes the `sqrt` operation.
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

    /// Executes the `exp` operation.
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

    /// Executes the `ln` operation.
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

    /// Executes the `log10` operation.
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

    /// Executes the `gamma` operation.
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

    /// Executes the `erf` operation.
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

    /// Executes the `erfc` operation.
    pub fn erfc(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(Self::real_erfc(*v))),
            Value::Complex(_) => Err(CalcError::TypeMismatch(
                "erfc currently supports real values only".to_string(),
            )),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "erfc",
                |v| Ok(Self::real_erfc(v)),
            )?)),
        })
    }

    /// Executes the `bessel` operation.
    pub fn bessel(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(Self::real_bessel_j0(*v))),
            Value::Complex(_) => Err(CalcError::TypeMismatch(
                "bessel currently supports real values only".to_string(),
            )),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "bessel",
                |v| Ok(Self::real_bessel_j0(v)),
            )?)),
        })
    }

    /// Executes the `mbessel` operation.
    pub fn mbessel(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(Self::real_modified_bessel_i0(*v))),
            Value::Complex(_) => Err(CalcError::TypeMismatch(
                "mbessel currently supports real values only".to_string(),
            )),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "mbessel",
                |v| Ok(Self::real_modified_bessel_i0(v)),
            )?)),
        })
    }

    /// Executes the `sinc` operation.
    pub fn sinc(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(Self::real_sinc(*v))),
            Value::Complex(c) => {
                let z = Self::to_complex64(*c);
                if z.norm() < 1e-12 {
                    Ok(Value::Real(1.0))
                } else {
                    Ok(Value::Complex(Self::from_complex64(z.sin() / z)))
                }
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry);
                if z.norm() < 1e-12 {
                    Ok(Complex { re: 1.0, im: 0.0 })
                } else {
                    Ok(Self::from_complex64(z.sin() / z))
                }
            })?)),
        })
    }

    /// Executes the `pow` operation.
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

    /// Executes the `percent` operation.
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

    /// Executes the `inv` operation.
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

    /// Executes the `square` operation.
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

    /// Executes the `root` operation.
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

    /// Executes the `exp10` operation.
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

    /// Executes the `exp2` operation.
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

    /// Executes the `log2` operation.
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

    /// Executes the `log_y_x` operation.
    pub fn log_y_x(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| {
            if let Some(value) =
                Self::matrix_elementwise_binary(left, right, "log_y_x", |lhs, rhs| {
                    let ln_base = lhs.ln();
                    if ln_base.norm() < 1e-12 {
                        return Err(CalcError::DomainError(
                            "log_y_x base produces an undefined logarithm".to_string(),
                        ));
                    }
                    Ok(rhs.ln() / ln_base)
                })?
            {
                return Ok(value);
            }
            match (left, right) {
                (Value::Real(base), Value::Real(x)) => {
                    if *base <= 0.0 || *base == 1.0 || *x <= 0.0 {
                        return Err(CalcError::DomainError(
                            "log_y_x requires real values with base > 0, base != 1, and x > 0"
                                .to_string(),
                        ));
                    }
                    Ok(Value::Real(x.log(*base)))
                }
                _ => {
                    let base = Self::as_complex(left, "log_y_x")?;
                    let x = Self::as_complex(right, "log_y_x")?;
                    let ln_base = Self::to_complex64(base).ln();
                    if ln_base.norm() < 1e-12 {
                        return Err(CalcError::DomainError(
                            "log_y_x base produces an undefined logarithm".to_string(),
                        ));
                    }
                    let out = Self::to_complex64(x).ln() / ln_base;
                    Ok(Value::Complex(Self::from_complex64(out)))
                }
            }
        })
    }

    /// Executes the `signum` operation.
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


}
