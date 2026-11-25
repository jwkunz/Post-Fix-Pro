//! Calculator operations for the statistics panel panel.

use super::*;

impl Calculator {
    /// Executes the `mean` operation.
    pub fn mean(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_mean, "mean")
    }

    /// Executes the `mode` operation.
    pub fn mode(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_mode, "mode")
    }

    /// Executes the `variance` operation.
    pub fn variance(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_variance_population, "variance")
    }

    /// Executes the `std_dev_p` operation.
    pub fn std_dev_p(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_std_dev_population, "std_dev_p")
    }

    /// Executes the `std_dev_s` operation.
    pub fn std_dev_s(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_std_dev_sample, "std_dev_s")
    }

    /// Executes the `median` operation.
    pub fn median(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_median, "median")
    }

    /// Executes the `quart` operation.
    pub fn quart(&mut self) -> Result<(), CalcError> {
        self.require_stack_len(1)?;
        let len = self.state.stack.len();
        match self.state.stack.last() {
            Some(Value::Matrix(matrix)) => {
                let summary = Self::matrix_quartiles_summary(matrix)?;
                self.state.stack[len - 1] = Value::Matrix(summary);
                Ok(())
            }
            Some(_) => {
                let scalar_vector = Self::stack_real_scalars_as_vector(&self.state.stack, "quart")?;
                let summary = Self::matrix_quartiles_summary(&scalar_vector)?;
                self.state.stack.clear();
                self.state.stack.push(Value::Matrix(summary));
                Ok(())
            }
            None => unreachable!("prechecked non-empty stack"),
        }
    }

    /// Executes the `max_value` operation.
    pub fn max_value(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_max, "max")
    }

    /// Executes the `min_value` operation.
    pub fn min_value(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_min, "min")
    }

}
