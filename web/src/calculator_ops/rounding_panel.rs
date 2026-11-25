//! Calculator operations for the rounding panel panel.

use super::*;

impl Calculator {
    /// Executes the `round_value` operation.
    pub fn round_value(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.round())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "round",
                |v| Ok(v.round()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "rnd currently supports real values only".to_string(),
            )),
        })
    }

    /// Executes the `floor_value` operation.
    pub fn floor_value(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.floor())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "floor",
                |v| Ok(v.floor()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "floor currently supports real values only".to_string(),
            )),
        })
    }

    /// Executes the `ceil_value` operation.
    pub fn ceil_value(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v.ceil())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "ceil",
                |v| Ok(v.ceil()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "ceil currently supports real values only".to_string(),
            )),
        })
    }

    /// Executes the `dec_part` operation.
    pub fn dec_part(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => Ok(Value::Real(v - v.trunc())),
            Value::Matrix(matrix) => Ok(Value::Matrix(Self::map_matrix_real_entries(
                matrix,
                "decP",
                |v| Ok(v - v.trunc()),
            )?)),
            _ => Err(CalcError::TypeMismatch(
                "decP currently supports real values only".to_string(),
            )),
        })
    }

}
