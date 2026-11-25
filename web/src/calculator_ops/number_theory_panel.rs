//! Calculator operations for the number theory panel panel.

use super::*;

impl Calculator {
    /// Executes the `factorial` operation.
    pub fn factorial(&mut self) -> Result<(), CalcError> {
        self.apply_unary_op(|value| match value {
            Value::Real(v) => {
                let n = Self::as_non_negative_integer(*v, "factorial")?;
                let mut out = 1.0;
                for i in 2..=n {
                    out *= i as f64;
                }
                Ok(Value::Real(out))
            }
            _ => Err(CalcError::TypeMismatch(
                "factorial currently supports real values only".to_string(),
            )),
        })
    }

    /// Executes the `ncr` operation.
    pub fn ncr(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(n), Value::Real(r)) => {
                let n = Self::as_non_negative_integer(*n, "nCr n")?;
                let r = Self::as_non_negative_integer(*r, "nCr r")?;
                if r > n {
                    return Err(CalcError::DomainError("nCr requires n >= r".to_string()));
                }
                Ok(Value::Real(Self::ncr_value(n, r)))
            }
            _ => Err(CalcError::TypeMismatch(
                "nCr currently supports real values only".to_string(),
            )),
        })
    }

    /// Executes the `npr` operation.
    pub fn npr(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(n), Value::Real(r)) => {
                let n = Self::as_non_negative_integer(*n, "nPr n")?;
                let r = Self::as_non_negative_integer(*r, "nPr r")?;
                if r > n {
                    return Err(CalcError::DomainError("nPr requires n >= r".to_string()));
                }
                let mut out = 1.0;
                for i in 0..r {
                    out *= (n - i) as f64;
                }
                Ok(Value::Real(out))
            }
            _ => Err(CalcError::TypeMismatch(
                "nPr currently supports real values only".to_string(),
            )),
        })
    }

    /// Executes the `modulo` operation.
    pub fn modulo(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(x), Value::Real(y)) => {
                if *y == 0.0 {
                    return Err(CalcError::DivideByZero);
                }
                Ok(Value::Real(x.rem_euclid(*y)))
            }
            _ => Err(CalcError::TypeMismatch(
                "mod currently supports real values only".to_string(),
            )),
        })
    }

    /// Executes the `rand_num` operation.
    pub fn rand_num(&mut self) -> Result<(), CalcError> {
        let next = self.next_random();
        self.state.stack.push(Value::Real(next));
        Ok(())
    }

    /// Executes the `gcd` operation.
    pub fn gcd(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(a), Value::Real(b)) => {
                let a = Self::as_integer(*a, "gcd a")?;
                let b = Self::as_integer(*b, "gcd b")?;
                let g = Self::gcd_u64(a.unsigned_abs(), b.unsigned_abs());
                Ok(Value::Real(g as f64))
            }
            _ => Err(CalcError::TypeMismatch(
                "gcd currently supports real values only".to_string(),
            )),
        })
    }

    /// Executes the `lcm` operation.
    pub fn lcm(&mut self) -> Result<(), CalcError> {
        self.apply_binary_op(|left, right| match (left, right) {
            (Value::Real(a), Value::Real(b)) => {
                let a = Self::as_integer(*a, "lcm a")?;
                let b = Self::as_integer(*b, "lcm b")?;
                let a_abs = a.unsigned_abs();
                let b_abs = b.unsigned_abs();
                if a_abs == 0 || b_abs == 0 {
                    return Ok(Value::Real(0.0));
                }
                let g = Self::gcd_u64(a_abs, b_abs);
                let l = (a_abs / g).saturating_mul(b_abs);
                Ok(Value::Real(l as f64))
            }
            _ => Err(CalcError::TypeMismatch(
                "lcm currently supports real values only".to_string(),
            )),
        })
    }

}
