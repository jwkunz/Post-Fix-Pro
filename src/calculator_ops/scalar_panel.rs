use super::*;

impl Calculator {
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


}
