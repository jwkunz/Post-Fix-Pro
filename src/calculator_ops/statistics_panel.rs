use super::*;

impl Calculator {
    pub fn mean(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_mean, "mean")
    }

    pub fn mode(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_mode, "mode")
    }

    pub fn variance(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_variance, "variance")
    }

    pub fn std_dev(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_std_dev, "std_dev")
    }

    pub fn max_value(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_max, "max")
    }

    pub fn min_value(&mut self) -> Result<(), CalcError> {
        self.apply_stat_op(Self::matrix_min, "min")
    }

}
