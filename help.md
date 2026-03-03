# Post Fix Pro Help

This help file matches the current GUI labels/tooltips.

Reverse Polish Notation (RPN): push values first, then apply operations.
For binary operations, unless otherwise stated:

- argument order: lower stack value is `x`, top stack value is `y`.

## Global

- `help`: open help dialog.
- `close`: close help dialog.
- `undo`: restore the state before the last successful operation.

## Scalar Keypad

- `deg_rad`: toggle angle mode.
- `pick`: consume top integer `n`; copy stack item `#n` to top.
- `clear_entry`: clear scalar entry buffer.
- `clear_all`: clear entry and full stack.
- `drop`: remove top value.
- `dup`: duplicate top value.
- `swap`: swap top two values.
- `rot`: rotate top three values.
- `+/-`: toggle sign in scalar entry.
- `0..9`: append digit to scalar entry.
- `.`: append decimal point.
- `entry_exp`: insert `E` in scalar entry.
- `enter`: push scalar entry; if empty, duplicate top.

Arithmetic:

- `add`: `x + y`.
- `sub`: `x - y`.
- `mul`: `x * y`.
- `div`: `x / y`.

## Matrix Builder

- `apply_size`: rebuild Matrix Entry from row/column controls.
- `push_matrix`: parse Matrix Entry and push matrix.
- `preset_matrix`: load matrix preset into Matrix Entry.
- `push_identity`: load identity matrix template `I(n)` into Matrix Entry.
- `import_csv`: import CSV into Matrix Entry.
- `export_csv`: export top matrix as CSV.
- `hstack`: top value is integer count `n`; consume `n` values and concatenate horizontally.
- `vstack`: top value is integer count `n`; consume `n` values and concatenate vertically.
- `hravel`: matrix -> split into columns on stack; vector -> split to scalar entries.
- `vravel`: matrix -> split into rows on stack; vector -> split to scalar entries.

## Vector Operators

- `dot`: dot product.
  - arg order: vectors `x`, `y` -> `x·y`.
- `cross`: cross product (3-vectors).
  - arg order: vectors `x`, `y` -> `x×y`.
- `diag`: vector -> diagonal matrix.
- `tpltz`: vector -> Toeplitz matrix.

## Matrix Operators

- `solve_ax_b`: solve linear system.
  - arg order: matrix `A`, matrix `B` -> solve `A*x = B`.
- `solve_lstsq`: least-squares solve (pseudoinverse based).
  - arg order: matrix `A`, matrix `B` -> `x = A^+ B`.
- `determinant`: determinant of top matrix.
- `trace`: trace of top matrix.
- `transpose`: transpose of top matrix.
- `inverse`: inverse of top matrix.
- `hadamard_mul`: element-wise multiply.
  - arg order: `x .* y`.
- `hadamard_div`: element-wise divide.
  - arg order: `x ./ y`.
- `norm_p`: p-norm of matrix/vector.
  - arg order: value `x`, scalar `p` -> `||x||_p`.
- `mat_exp`: matrix exponential `e^A`.
- `hermitian`: conjugate transpose `A^H`.
- `mat_pow`: integer matrix power.
  - arg order: matrix `A`, integer `n` -> `A^n`.

## Matrix Decompositions

- `qr`: QR decomposition (`Q`, `R`).
- `lu`: LU decomposition with pivoting (`P`, `L`, `U`).
- `svd`: singular value decomposition (`U`, `S`, `Vt`).
- `evd`: eigendecomposition (or Schur fallback with warning).

## Scalar Panel

- `neg`: unary negation.
- `inv`: reciprocal `1/x`.
- `square`: `x^2`.
- `sqrt`: square root.
- `pow`: power.
  - arg order: `x`, `y` -> `x^y`.
- `root`: y-th root.
  - arg order: `x`, `y` -> `x^(1/y)`.
- `exp10`: `10^x`.
- `log10`: `log10(x)`.
- `exp`: `e^x`.
- `ln`: `ln(x)`.
- `exp2`: `2^x`.
- `log2`: `log2(x)`.
- `log_y_x`: arbitrary-base log.
  - arg order: base `x`, value `y` -> `log_x(y)`.
- `signum`: signum.
- `percent`: percentage-of.
  - arg order: base `x`, percent `y` -> `x*y/100`.

## Trigonometry Panel

- `hyp_toggle`: toggle circular/hyperbolic mode.
- `to_rad`: convert degrees -> radians.
- `to_deg`: convert radians -> degrees.
- `atan2`: two-argument arctangent.
  - arg order: `x`, `y` -> `atan2(y, x)`.
- `sin`: sine.
- `asin`: inverse sine.
- `cos`: cosine.
- `acos`: inverse cosine.
- `tan`: tangent.
- `atan`: inverse tangent.
- `sec`: secant.
- `asec`: inverse secant.
- `csc`: cosecant.
- `acsc`: inverse cosecant.
- `cot`: cotangent.
- `acot`: inverse cotangent.

## Complex Panel

- `abs`: magnitude.
- `abs_sq`: squared magnitude.
- `arg`: phase angle.
- `conjugate`: complex conjugate.
- `real_part`: real component.
- `imag_part`: imaginary component.
- `cart`: rectangular compose/decompose.
  - arg order (compose): `a`, `b` -> `a + bi`.
- `pol`: polar compose/decompose.
  - arg order (compose): `r`, `theta`.
- `npol`: normalized polar compose/decompose.
  - arg order (compose): `r`, `cycles` where `theta = 2*pi*cycles`.

## Special Panel

- `factorial`: factorial `n!`.
- `ncr`: combinations.
  - arg order: `n`, `r` -> `nCr`.
- `npr`: permutations.
  - arg order: `n`, `r` -> `nPr`.
- `modulo`: Euclidean modulo.
  - arg order: `x`, `y` -> `x mod y`.
- `gcd`: greatest common divisor.
  - arg order: `x`, `y` -> `gcd(x,y)`.
- `lcm`: least common multiple.
  - arg order: `x`, `y` -> `lcm(x,y)`.
- `gamma`: gamma function `Gamma(x)`.
- `erf`: error function.
- `erfc`: complementary error function.
- `bessel`: Bessel J0.
- `mbessel`: modified Bessel I0.
- `sinc`: `sin(x)/x` with limit at zero.

## Statistics Panel

- `mean`: mean of sample vector.
- `mode`: mode of sample vector.
- `median`: median of sample vector.
- `quart`: quartile summary `[min, q1, q2, q3, max]`.
- `std_dev_p`: population standard deviation.
- `std_dev_s`: sample standard deviation.
- `variance`: population variance.
- `max`: maximum of sample vector.
- `min`: minimum of sample vector.
- `rand_num`: pseudo-random scalar in `[0, 1)`.

## Rounding Panel

- `round`: nearest integer.
- `floor`: floor.
- `ceil`: ceiling.
- `dec_part`: decimal part (`x - trunc(x)`).

## Constants Panel

- `pi`: push pi.
- `e`: push Euler's number.
- `γ`: push Euler-Mascheroni constant.
- `ψ`: push golden ratio.
- `c`: push speed of light constant.
- `mol`: push Avogadro constant.
- `k`: push Boltzmann constant.
- `hbar`: push reduced Planck constant.
- `epsilon_0`: push vacuum permittivity.
- `mu_0`: push vacuum permeability.
- `g`: push Newtonian gravitational constant.
- `q_e`: push electron charge.

## Memory Panel

- `memory_store`: store top value in selected register.
- `memory_recall`: recall selected register to top of stack.
- `memory_clear`: clear selected register.
