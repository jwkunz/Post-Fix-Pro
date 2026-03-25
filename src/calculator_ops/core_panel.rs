//! Calculator operations for the core panel panel.

use super::*;

impl Calculator {
    /// Executes the `set_angle_mode` operation.
    pub fn set_angle_mode(&mut self, mode: AngleMode) {
        self.state.angle_mode = mode;
    }

    /// Executes the `set_display_mode` operation.
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.state.display_mode = mode;
    }

    /// Executes the `set_precision` operation.
    pub fn set_precision(&mut self, precision: u8) {
        self.state.precision = precision.min(12);
    }

    /// Executes the `push_pi` operation.
    pub fn push_pi(&mut self) {
        self.state.stack.push(Value::Real(std::f64::consts::PI));
    }

    /// Executes the `push_e` operation.
    pub fn push_e(&mut self) {
        self.state.stack.push(Value::Real(std::f64::consts::E));
    }

    /// Executes the `entry_set` operation.
    pub fn entry_set(&mut self, value: &str) {
        self.state.entry_buffer = value.to_string();
    }

    /// Executes the `clear_entry` operation.
    pub fn clear_entry(&mut self) {
        self.state.entry_buffer.clear();
    }

    /// Executes the `clear_all` operation.
    pub fn clear_all(&mut self) {
        self.state.stack.clear();
        self.state.entry_buffer.clear();
    }

    /// Executes the `push_value` operation.
    pub fn push_value(&mut self, value: Value) {
        self.state.stack.push(value);
    }

    /// Executes the `enter` operation.
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
