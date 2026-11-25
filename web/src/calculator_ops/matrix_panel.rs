//! Calculator operations for the matrix panel panel.

use super::*;

impl Calculator {
    /// Executes the `transpose` operation.
    pub fn transpose(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_transpose(matrix))),
            _ => Err(CalcError::TypeMismatch(
                "transpose requires a matrix value".to_string(),
            )),
        })
    }

    /// Executes the `push_identity` operation.
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

    /// Executes the `stack_vec` operation.
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

    /// Executes the `hstack` operation.
    pub fn hstack(&mut self) -> Result<(), CalcError> {
        self.stack_combine(true)
    }

    /// Executes the `vstack` operation.
    pub fn vstack(&mut self) -> Result<(), CalcError> {
        self.stack_combine(false)
    }

    /// Executes the `ravel` operation.
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

    /// Executes the `hravel` operation.
    pub fn hravel(&mut self) -> Result<(), CalcError> {
        self.matrix_ravel(true)
    }

    /// Executes the `vravel` operation.
    pub fn vravel(&mut self) -> Result<(), CalcError> {
        self.matrix_ravel(false)
    }

    /// Executes the `determinant` operation.
    pub fn determinant(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Complex(Self::matrix_determinant(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "determinant requires a matrix value".to_string(),
            )),
        })
    }

    /// Executes the `inverse` operation.
    pub fn inverse(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_inverse(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "inverse requires a matrix value".to_string(),
            )),
        })
    }

    /// Executes the `solve_ax_b` operation.
    pub fn solve_ax_b(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_solve(a, b)?)),
            _ => Err(CalcError::TypeMismatch(
                "solve_ax_b requires two matrix operands (A then B)".to_string(),
            )),
        })
    }

    /// Executes the `solve_lstsq` operation.
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

    /// Executes the `dot` operation.
    pub fn dot(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Complex(Self::matrix_dot(a, b)?)),
            _ => Err(CalcError::TypeMismatch(
                "dot requires two vector matrices".to_string(),
            )),
        })
    }

    /// Executes the `cross` operation.
    pub fn cross(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Matrix(a), Value::Matrix(b)) => Ok(Value::Matrix(Self::matrix_cross(a, b)?)),
            _ => Err(CalcError::TypeMismatch(
                "cross requires two 3-element vector matrices".to_string(),
            )),
        })
    }

    /// Executes the `trace` operation.
    pub fn trace(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Complex(Self::matrix_trace(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "trace requires a matrix value".to_string(),
            )),
        })
    }

    /// Executes the `norm_p` operation.
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

    /// Executes the `diag` operation.
    pub fn diag(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_diag(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "diag requires a vector matrix value".to_string(),
            )),
        })
    }

    /// Executes the `toep` operation.
    pub fn toep(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_toeplitz(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "toep requires a vector matrix value".to_string(),
            )),
        })
    }

    /// Executes the `mat_exp` operation.
    pub fn mat_exp(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_exp(matrix)?)),
            _ => Err(CalcError::TypeMismatch(
                "MatExp requires a matrix value".to_string(),
            )),
        })
    }

    /// Executes the `hermitian` operation.
    pub fn hermitian(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::matrix_hermitian(matrix))),
            _ => Err(CalcError::TypeMismatch(
                "Hermitian requires a matrix value".to_string(),
            )),
        })
    }

    /// Executes the `mat_pow` operation.
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

    /// Executes the `qr` operation.
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

    /// Executes the `lu` operation.
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

    /// Executes the `svd` operation.
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

    /// Executes the `evd` operation.
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

}
