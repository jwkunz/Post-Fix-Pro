//! Calculator operations for the complex panel panel.

use super::*;

impl Calculator {
    /// Executes the `abs` operation.
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

    /// Executes the `abs_sq` operation.
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

    /// Executes the `arg` operation.
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

    /// Executes the `conjugate` operation.
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

    /// Executes the `real_part` operation.
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

    /// Executes the `imag_part` operation.
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

    /// Executes the `cart` operation.
    pub fn cart(&mut self) -> Result<(), CalcError> {
        self.complex_stack_transform(ComplexTransformMode::Cartesian)
    }

    /// Executes the `pol` operation.
    pub fn pol(&mut self) -> Result<(), CalcError> {
        self.complex_stack_transform(ComplexTransformMode::Polar)
    }

    /// Executes the `npol` operation.
    pub fn npol(&mut self) -> Result<(), CalcError> {
        self.complex_stack_transform(ComplexTransformMode::NormalizedPolar)
    }

    /// Executes the `atan2` operation.
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


}
