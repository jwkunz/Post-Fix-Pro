//! Calculator operations for the memory panel panel.

use super::*;

impl Calculator {
    /// Executes the `memory_store` operation.
    pub fn memory_store(&mut self, register: usize) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let index = Self::validate_register(register)?;
        self.state.memory[index] = self.state.stack.last().cloned();
        Ok(())
    }

    /// Executes the `memory_recall` operation.
    pub fn memory_recall(&mut self, register: usize) -> Result<(), CalcError> {
        let index = Self::validate_register(register)?;
        let value = self.state.memory[index]
            .clone()
            .ok_or(CalcError::EmptyRegister(register))?;
        self.state.stack.push(value);
        Ok(())
    }

    /// Executes the `memory_clear` operation.
    pub fn memory_clear(&mut self, register: usize) -> Result<(), CalcError> {
        let index = Self::validate_register(register)?;
        self.state.memory[index] = None;
        Ok(())
    }

}
