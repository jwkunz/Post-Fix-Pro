use super::*;

impl Calculator {
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

}
