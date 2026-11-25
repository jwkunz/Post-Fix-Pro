//! Core domain types shared by the calculator engine and API layer.

/// Stack value type used by the RPN engine.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Real scalar value.
    Real(f64),
    /// Complex scalar value.
    Complex(Complex),
    /// Dense matrix value.
    Matrix(Matrix),
}

/// Complex scalar in Cartesian form (`re + i*im`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    /// Real component.
    pub re: f64,
    /// Imaginary component.
    pub im: f64,
}

/// Row-major dense matrix with complex entries.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    /// Number of rows.
    pub rows: usize,
    /// Number of columns.
    pub cols: usize,
    /// Row-major complex elements with length `rows * cols`.
    pub data: Vec<Complex>,
}

impl Matrix {
    /// Creates a new matrix after validating dimensions and data length.
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

/// Angle interpretation mode for trigonometric operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleMode {
    /// Degrees.
    Deg,
    /// Radians.
    Rad,
}

/// Numeric display mode for frontend formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Fixed-point format.
    Fix,
    /// Scientific notation.
    Sci,
    /// Engineering notation.
    Eng,
}

/// Internal complex transform encoding for cart/pol/npol conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComplexTransformMode {
    Cartesian,
    Polar,
    NormalizedPolar,
}

/// Error conditions returned by calculator operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    /// Operation needs more stack values than currently available.
    StackUnderflow { needed: usize, available: usize },
    /// User input was syntactically valid but semantically rejected.
    InvalidInput(String),
    /// Shape mismatch in matrix/vector operations.
    DimensionMismatch { expected: usize, actual: usize },
    /// Value type did not satisfy operation requirements.
    TypeMismatch(String),
    /// Memory register index was out of bounds.
    InvalidRegister(usize),
    /// Memory register was empty on recall.
    EmptyRegister(usize),
    /// Domain restriction violation (e.g., `ln(x<=0)` over reals).
    DomainError(String),
    /// Division by zero.
    DivideByZero,
    /// Matrix is singular or not invertible for requested operation.
    SingularMatrix(String),
}

/// Complete mutable calculator state.
#[derive(Debug, Clone, PartialEq)]
pub struct CalcState {
    /// Bottom-to-top stack representation.
    pub stack: Vec<Value>,
    /// Current scalar entry text buffer.
    pub entry_buffer: String,
    /// Active trigonometric angle mode.
    pub angle_mode: AngleMode,
    /// Active display mode.
    pub display_mode: DisplayMode,
    /// Decimal precision for formatted output.
    pub precision: u8,
    /// A-Z memory registers.
    pub memory: Vec<Option<Value>>,
    /// Internal deterministic RNG state.
    pub(crate) rng_state: u64,
}
