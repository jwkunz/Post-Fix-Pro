use super::*;

impl Calculator {
    pub fn memory_store(&mut self, register: usize) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let index = Self::validate_register(register)?;
        self.state.memory[index] = self.state.stack.last().cloned();
        Ok(())
    }

    pub fn memory_recall(&mut self, register: usize) -> Result<(), CalcError> {
        let index = Self::validate_register(register)?;
        let value = self.state.memory[index]
            .clone()
            .ok_or(CalcError::EmptyRegister(register))?;
        self.state.stack.push(value);
        Ok(())
    }

    pub fn memory_clear(&mut self, register: usize) -> Result<(), CalcError> {
        let index = Self::validate_register(register)?;
        self.state.memory[index] = None;
        Ok(())
    }

}
