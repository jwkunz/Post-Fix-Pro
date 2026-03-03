# Calculator Help

This calculator uses Reverse Polish Notation (RPN):

1. Type a value into the entry line.
2. Press `Enter` to push it onto the stack.
3. Apply operators to the top stack value(s).

Stack labels:

- `T`: top of stack
- `#n`: lower stack positions

## Scalar Keypad

- `RAD/DEG`: toggle angle mode between radians and degrees
- `Pick`: use top stack integer as stack line index (`#n`) and replace top with that copied value
- `CE`: clear current entry buffer
- `C`: clear stack + entry
- `Drop`: remove top stack value
- `Dup`: duplicate top stack value
- `Swap`: swap top two stack values
- `Rot`: rotate top three values
- `+/-`: toggle sign on current entry
- `/`, `*`, `-`, `+`: arithmetic
- `0..9`, `.`, `EXP`: number entry (`EXP` inserts `E` for scientific notation, e.g. `2.3E3`)
- `Enter`: push entry buffer onto stack

## Matrix Panels

- `Matrix Builder`: Matrix Entry box, sizing, Push, CSV import/export, and preset template
- `Vector Operators`: dot, cross, diag, and tpltz
- `Matrix Operators`: solve, det, trace, transpose, inverse, H mul, H div, norm_p, exp(mat), herm, mat x^y
- `Matrix Decompositions`: QR, LU, SVD, EVD

### Matrix Operations

- `Push`: parse matrix text and push to stack (use repeatedly to push multiple matrices)
- `hstack`: top-of-stack is integer count `n`; consumes `n` stack values and combines horizontally
  - scalars -> `1xN` row vector
  - equal-size row vectors -> wider row vector
  - equal-size column vectors -> matrix by column concatenation
  - equal-size matrices -> horizontal block concatenation
- `vstack`: same count-driven behavior as `hstack`, but combines vertically
  - scalars -> `Nx1` column vector
  - equal-size row vectors -> matrix by row concatenation
  - equal-size column vectors -> taller column vector
  - equal-size matrices -> vertical block concatenation
- `hravel`: if top is matrix, split into column vectors on stack; if top is vector, unpack entries to stack scalars
- `vravel`: if top is matrix, split into row vectors on stack; if top is vector, unpack entries to stack scalars
- `Import CSV -> A`: load a CSV file into the Matrix Entry text area and size controls
- `Export Top CSV`: write the top-of-stack matrix to a downloadable `matrix.csv` file
- `Size A` controls:
  - set rows + columns
  - press `Apply` to regenerate matrix text with that size
- `Solve A*x=B`: solve linear system using top two matrices
- `LSTSQ Solve`: least-squares solve using Moore-Penrose pseudoinverse (`x = A^+ * B`, equivalent to normal-equation solution with pseudoinverse handling)
  - returns a status message with residual norm
  - warns when system is rank-deficient and reports that a minimum-norm solution was returned
- `det`: determinant of top matrix
- `trace`: trace of top matrix
- `transpose`: transpose top matrix
- `inverse`: inverse of top matrix
- `dot`: vector dot product (supports `Nx1` and `1xN`)
- `cross`: vector cross product (3-element vectors)
- `H mul`: element-wise multiplication (matrix with same-shape matrix, or matrix with scalar)
- `H div`: element-wise division (matrix by same-shape matrix, matrix/scalar, or scalar/matrix)
- `norm_p`: p-norm (push matrix/vector, then push `p`, then press `norm_p`)
- `diag`: convert vector matrix (`Nx1` or `1xN`) into a diagonal matrix
- `tpltz`: convert vector matrix (`Nx1` or `1xN`) into a Toeplitz matrix using `T[i,j] = v[|i-j|]`
- `exp(mat)`: matrix exponential of top square matrix (`e^A`)
- `herm`: Hermitian (conjugate transpose) of top matrix
- `mat x^y`: integer matrix power (push matrix, then integer exponent, then press `MatPow`)
- `QR`: QR decomposition (supports complex matrices); replaces top matrix with `Q` and pushes `R`
- `LU`: LU decomposition with partial pivoting (supports complex matrices); replaces top matrix with `P` and pushes `L`, then `U` (so `P*A = L*U`)
- `SVD`: singular value decomposition (supports complex matrices); replaces top matrix with `U` and pushes `S`, then `Vt`
- `EVD`: eigendecomposition; replaces top matrix with `V` and pushes `D`. If exact diagonalization is unavailable, returns Schur form (`Q`, `T`) with a warning.
- Decomposition outputs are tagged in the stack view as `Q/R`, `P/L/U`, `U/S/Vt`, and `V/D` for quick identification.
- `Preset`: convenience template
- `Preset I(n)`: fill Matrix Entry with an identity matrix template of size `n`

Matrix input format:

- One row per line
- Values separated by spaces or commas
- Values are parsed when you press `Push` (free-form text allowed while typing)
- Scientific notation is supported in values (e.g. `-1.2E-3`)
- Matrix entries are cast to complex values on push (`x` becomes `x + 0i`)
- Complex matrix literals can be entered as `(re,im)` (example: `(1.5,-2)` or `(2E1,3E-2)`)
- Keyboard focus in matrix/complex text fields captures typing (calculator hotkeys are suppressed there)
- Use `Shift+Enter` for explicit newline entry while editing matrices

Example:

```text
1 2
3 4
```

## Scientific Panel

### Trig / Angle

- If top-of-stack is a matrix, trig/angle operators apply element-wise.
- `sin`, `asin`, `cos`, `acos`, `tan`, `atan`
- `atan2`: binary operation using stack order `(y, x)`
- `to rad`: convert top real value degrees -> radians
- `to deg`: convert top real value radians -> degrees
- `hyp`: toggles trig buttons to hyperbolic/inverse-hyperbolic variants:
  - `sinh`, `asinh`, `cosh`, `acosh`, `tanh`, `atanh`

### Powers / Logs / Core

- If top-of-stack is a matrix, scalar core operators apply element-wise.
- `inv x`: reciprocal (`1/x`)
- `x^2`: square
- `sqrt`: square root
- `x^y`: power (`x` then `y`)
- `x√y`: y-th root of x (`x` then `y`)
- `10^x`, `e^x`, `2^x`
- `log10`, `ln x`, `log2 x`
- `gamma`, `signum`
- `%`: percentage (`base * percent / 100`)
- `erf`
- `pi`, `e`: push constants

### Complex

- If top-of-stack is a matrix, complex operators apply element-wise.
- `abs`: magnitude
- `abs^2`: squared magnitude
- `arg`: phase/argument (respects RAD/DEG mode)
- `conj`: complex conjugate
- `real()`: real part of top scalar
- `imag()`: imaginary part of top scalar
- `cart`: two-way conversion
  - if top is complex: decomposes to `real`, `imag`
  - else if top two are real scalars: composes `a + bi`
- `pol`: two-way conversion
  - if top is complex: decomposes to `magnitude`, `arg` (RAD/DEG aware)
  - else if top two are real scalars: composes from `magnitude`, `arg` (RAD/DEG aware)
- `npol`: two-way conversion
  - if top is complex: decomposes to `magnitude`, `cycles` where `arg = 2π*cycles`
  - else if top two are real scalars: composes from `magnitude`, `cycles`

### Integer / Number Tools

- Rounding operators (`round`, `floor`, `ceil`, `dec part`) apply element-wise to matrices.
- `n!`: factorial (non-negative integer)
- `nCr`: combinations
- `nPr`: permutations
- `x mod y`: Euclidean remainder
- `GCD`: greatest common divisor (integers)
- `LCM`: least common multiple (integers)
- `round`: round to nearest integer
- `floor`: floor
- `ceil`: ceil
- `dec part`: decimal part (`x - trunc(x)`)

### Statistics

- Works on either: top vector matrix (`Nx1` or `1xN`) or a scalar-only stack (treated as one set).
- `ran#`: pseudo-random number in `[0, 1)`
- `mean`: arithmetic mean of a real vector (`Nx1` or `1xN`)
- `mode`: most frequent value in a real vector
- `std dev`: population standard deviation of a real vector
- `variance`: population variance of a real vector
- `max`: maximum value of a real vector
- `min`: minimum value of a real vector

### Constants Panel

- `pi`: π
- `e`: Euler's number
- `gamma`: Euler-Mascheroni constant `0.5772156649015329`
- `ψ`: golden ratio `1.618033988749895`
- `c`: speed of light `299792458`
- `mol`: Avogadro constant `6.02214076E23`
- `k`: Boltzmann constant `1.380649E-23`
- `hbar`: reduced Planck constant `1.054571817E-34`
- `epsilon_0`: vacuum permittivity `8.8541878128E-12`
- `mu_0`: vacuum permeability `1.25663706212E-6`
- `G`: Newtonian gravitational constant `6.67430E-11`

## Memory (A-Z)

- Choose register letter in input (`A` to `Z`)
- `STO`: store top stack value
- `RCL`: recall register onto stack
- `CLR`: clear register

## Keyboard Shortcuts

- `0-9`: append digit to entry
- `.`: decimal point
- `e`/`E`: exponent marker in scalar entry
- `Enter`: push entry
- `Backspace`: remove one character from entry
- `+`, `-`, `*`, `/`: arithmetic
- `Delete`: `Drop`
- `d`: `Dup`
- `s`: `Swap`
- `r` then digits then `Enter`/`Space`: `roll(n)`
- `p` then digits then `Enter`/`Space`: `pick(n)`
- `Esc`: cancel pending `r`/`p` sequence or close Help dialog

## Error Handling

Invalid operations do not mutate the stack.  
Examples:

- stack underflow (not enough operands)
- divide by zero
- domain errors (`ln(<=0)`, `sqrt(<0)` for real mode, etc.)
- type mismatch (matrix/scalar incompatibility)
