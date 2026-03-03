use super::*;

impl Calculator {
    pub fn set_angle_mode(&mut self, mode: AngleMode) {
        self.state.angle_mode = mode;
    }

    pub fn push_pi(&mut self) {
        self.state.stack.push(Value::Real(std::f64::consts::PI));
    }

    pub fn push_e(&mut self) {
        self.state.stack.push(Value::Real(std::f64::consts::E));
    }

    pub fn entry_set(&mut self, value: &str) {
        self.state.entry_buffer = value.to_string();
    }

    pub fn clear_entry(&mut self) {
        self.state.entry_buffer.clear();
    }

    pub fn clear_all(&mut self) {
        self.state.stack.clear();
        self.state.entry_buffer.clear();
    }

    pub fn push_value(&mut self, value: Value) {
        self.state.stack.push(value);
    }

    pub fn enter(&mut self) -> Result<(), CalcError> {
        if self.state.entry_buffer.trim().is_empty() {
            return Err(CalcError::InvalidInput("entry buffer is empty".to_string()));
        }

        let value = self.state.entry_buffer.parse::<f64>().map_err(|_| {
            CalcError::InvalidInput("entry buffer is not a valid number".to_string())
        })?;

        self.state.stack.push(Value::Real(value));
        self.state.entry_buffer.clear();
        Ok(())
    }

}
