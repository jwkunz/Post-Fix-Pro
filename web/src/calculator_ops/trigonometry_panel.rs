//! Calculator operations for the trigonometry panel panel.

use super::*;

impl Calculator {
    /// Executes the `sin` operation.
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

    /// Executes the `cos` operation.
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

    /// Executes the `tan` operation.
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

    /// Executes the `asin` operation.
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

    /// Executes the `acos` operation.
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

    /// Executes the `atan` operation.
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

    /// Executes the `sec` operation.
    pub fn sec(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let radians = match mode {
                    AngleMode::Deg => v.to_radians(),
                    AngleMode::Rad => *v,
                };
                let cos = radians.cos();
                if cos.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(1.0 / cos))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c).cos();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64(Complex64::new(1.0, 0.0) / z)))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry).cos();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64(Complex64::new(1.0, 0.0) / z))
            })?)),
        })
    }

    /// Executes the `csc` operation.
    pub fn csc(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let radians = match mode {
                    AngleMode::Deg => v.to_radians(),
                    AngleMode::Rad => *v,
                };
                let sin = radians.sin();
                if sin.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(1.0 / sin))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c).sin();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64(Complex64::new(1.0, 0.0) / z)))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry).sin();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64(Complex64::new(1.0, 0.0) / z))
            })?)),
        })
    }

    /// Executes the `cot` operation.
    pub fn cot(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let radians = match mode {
                    AngleMode::Deg => v.to_radians(),
                    AngleMode::Rad => *v,
                };
                let tan = radians.tan();
                if tan.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(1.0 / tan))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c).tan();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64(Complex64::new(1.0, 0.0) / z)))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry).tan();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64(Complex64::new(1.0, 0.0) / z))
            })?)),
        })
    }

    /// Executes the `asec` operation.
    pub fn asec(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                if v.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                let radians = (1.0 / v).acos();
                let out = match mode {
                    AngleMode::Deg => radians.to_degrees(),
                    AngleMode::Rad => radians,
                };
                Ok(Value::Real(out))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64((Complex64::new(1.0, 0.0) / z).acos())))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64((Complex64::new(1.0, 0.0) / z).acos()))
            })?)),
        })
    }

    /// Executes the `acsc` operation.
    pub fn acsc(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                if v.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                let radians = (1.0 / v).asin();
                let out = match mode {
                    AngleMode::Deg => radians.to_degrees(),
                    AngleMode::Rad => radians,
                };
                Ok(Value::Real(out))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64((Complex64::new(1.0, 0.0) / z).asin())))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64((Complex64::new(1.0, 0.0) / z).asin()))
            })?)),
        })
    }

    /// Executes the `acot` operation.
    pub fn acot(&mut self) -> Result<(), CalcError> {
        let mode = self.state.angle_mode;
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let radians = if v.abs() < 1e-12 {
                    std::f64::consts::FRAC_PI_2
                } else {
                    (1.0 / v).atan()
                };
                let out = match mode {
                    AngleMode::Deg => radians.to_degrees(),
                    AngleMode::Rad => radians,
                };
                Ok(Value::Real(out))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c);
                if z.norm() < 1e-12 {
                    return Ok(Value::Real(std::f64::consts::FRAC_PI_2));
                }
                Ok(Value::Complex(Self::from_complex64((Complex64::new(1.0, 0.0) / z).atan())))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry);
                if z.norm() < 1e-12 {
                    return Ok(Complex {
                        re: std::f64::consts::FRAC_PI_2,
                        im: 0.0,
                    });
                }
                Ok(Self::from_complex64((Complex64::new(1.0, 0.0) / z).atan()))
            })?)),
        })
    }

    /// Executes the `sinh` operation.
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

    /// Executes the `cosh` operation.
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

    /// Executes the `tanh` operation.
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

    /// Executes the `asinh` operation.
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

    /// Executes the `acosh` operation.
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

    /// Executes the `atanh` operation.
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

    /// Executes the `sech` operation.
    pub fn sech(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(1.0 / v.cosh())),
            Value::Complex(c) => {
                let z = Self::to_complex64(*c).cosh();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64(Complex64::new(1.0, 0.0) / z)))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry).cosh();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64(Complex64::new(1.0, 0.0) / z))
            })?)),
        })
    }

    /// Executes the `csch` operation.
    pub fn csch(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let s = v.sinh();
                if s.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(1.0 / s))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c).sinh();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64(Complex64::new(1.0, 0.0) / z)))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry).sinh();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64(Complex64::new(1.0, 0.0) / z))
            })?)),
        })
    }

    /// Executes the `coth` operation.
    pub fn coth(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let t = v.tanh();
                if t.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(1.0 / t))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c).tanh();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64(Complex64::new(1.0, 0.0) / z)))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry).tanh();
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64(Complex64::new(1.0, 0.0) / z))
            })?)),
        })
    }

    /// Executes the `asech` operation.
    pub fn asech(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                if v.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real((1.0 / v).acosh()))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64((Complex64::new(1.0, 0.0) / z).acosh())))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64((Complex64::new(1.0, 0.0) / z).acosh()))
            })?)),
        })
    }

    /// Executes the `acsch` operation.
    pub fn acsch(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                if v.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real((1.0 / v).asinh()))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64((Complex64::new(1.0, 0.0) / z).asinh())))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64((Complex64::new(1.0, 0.0) / z).asinh()))
            })?)),
        })
    }

    /// Executes the `acoth` operation.
    pub fn acoth(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                if v.abs() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real((1.0 / v).atanh()))
            }
            Value::Complex(c) => {
                let z = Self::to_complex64(*c);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Complex(Self::from_complex64((Complex64::new(1.0, 0.0) / z).atanh())))
            }
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_entries(matrix, |entry| {
                let z = Self::to_complex64(entry);
                if z.norm() < 1e-12 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Self::from_complex64((Complex64::new(1.0, 0.0) / z).atanh()))
            })?)),
        })
    }

    /// Executes the `to_rad` operation.
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

    /// Executes the `to_deg` operation.
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

}
