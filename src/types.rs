#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Real(f64),
    Complex(Complex),
    Matrix(Matrix),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Complex>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<Complex>) -> Result<Self, CalcError> {
        if rows == 0 || cols == 0 {
            return Err(CalcError::InvalidInput(
                "matrix dimensions must be non-zero".to_string(),
            ));
        }

        if rows * cols != data.len() {
            return Err(CalcError::DimensionMismatch {
                expected: rows * cols,
                actual: data.len(),
            });
        }

        Ok(Self { rows, cols, data })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleMode {
    Deg,
    Rad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Fix,
    Sci,
    Eng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComplexTransformMode {
    Cartesian,
    Polar,
    NormalizedPolar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    StackUnderflow { needed: usize, available: usize },
    InvalidInput(String),
    DimensionMismatch { expected: usize, actual: usize },
    TypeMismatch(String),
    InvalidRegister(usize),
    EmptyRegister(usize),
    DomainError(String),
    DivideByZero,
    SingularMatrix(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcState {
    pub stack: Vec<Value>,
    pub entry_buffer: String,
    pub angle_mode: AngleMode,
    pub display_mode: DisplayMode,
    pub precision: u8,
    pub memory: Vec<Option<Value>>,
    pub(crate) rng_state: u64,
}
