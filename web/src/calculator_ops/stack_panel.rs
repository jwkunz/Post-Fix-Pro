//! Calculator operations for the stack panel panel.

use super::*;

impl Calculator {
    /// Executes the `drop` operation.
    pub fn drop(&mut self) -> Result<Value, CalcError> {
        self.require_stack_len(1)?;
        Ok(self.state.stack.pop().expect("prechecked stack length"))
    }

    /// Executes the `dup` operation.
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

    /// Executes the `swap` operation.
    pub fn swap(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(2)?;
        let len = self.state.stack.len();
        self.state.stack.swap(len - 1, len - 2);
        Ok(())
    }

    /// Executes the `rot` operation.
    pub fn rot(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(3)?;
        let len = self.state.stack.len();
        self.state.stack[len - 3..].rotate_left(1);
        Ok(())
    }

    /// Executes the `roll` operation.
    pub fn roll(&mut self, count: usize) -> Result<(), CalcError> {
        if count < 2 {
            return Err(CalcError::InvalidInput(
                "roll count must be at least 2".to_string(),
            ));
        }
        self.require_stack_len(count)?;
        let len = self.state.stack.len();
        self.state.stack[len - count..].rotate_left(1);
        Ok(())
    }

    /// Executes the `pick` operation.
    pub fn pick(&mut self, depth: usize) -> Result<(), CalcError> {
        if depth == 0 {
            return Err(CalcError::InvalidInput(
                "pick depth must be at least 1".to_string(),
            ));
        }
        self.require_stack_len(depth)?;
        let len = self.state.stack.len();
        let value = self.state.stack[len - depth].clone();
        self.state.stack.push(value);
        Ok(())
    }

    /// Executes the `pick_from_stack_index` operation.
    pub fn pick_from_stack_index(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(2)?;
        let len = self.state.stack.len();
        let index = match self.state.stack.get(len - 1) {
            Some(Value::Real(v)) => Self::as_non_negative_integer(*v, "pick index")? as usize,
            Some(Value::Complex(c)) if c.im.abs() <= 1e-12 => {
                Self::as_non_negative_integer(c.re, "pick index")? as usize
            }
            _ => {
                return Err(CalcError::TypeMismatch(
                    "pick index must be a non-negative integer scalar".to_string(),
                ));
            }
        };

        if index >= len - 1 {
            return Err(CalcError::InvalidInput(format!(
                "pick index out of range: {index} (stack lines are 0..{})",
                len - 2
            )));
        }

        let value = self.state.stack[index].clone();
        self.state.stack[len - 1] = value;
        Ok(())
    }

}
