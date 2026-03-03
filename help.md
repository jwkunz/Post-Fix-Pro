# Post Fix Pro Help

This calculator uses Reverse Polish Notation (RPN): push values first, then apply operations.
For any binary operation below, unless otherwise noted:

- `arg order`: top of stack is the **right** argument `y`, next below is the **left** argument `x`.

## Stack

- `t`: top stack slot marker in the stack panel.
- `#n`: lower stack slot index marker in the stack panel.
- `visible`: number of stack frames shown (1..64).

## Top Bar / Dialog

- `help`: open the help dialog.
- `close`: close the help dialog.

## Scalar Keypad Panel

- `rad/deg`: toggle angle mode.
- `deg_rad`: toggle angle mode.
- `pick`: consume top integer `n`; duplicate stack item `#n` to the top.
- `ce`: clear scalar entry buffer only.
- `clear_entry`: clear scalar entry buffer only.
- `c`: clear entry and full stack.
- `clear_all`: clear entry and full stack.
- `drop`: remove top stack value.
- `dup`: duplicate top stack value.
- `swap`: swap top two stack values.
- `rot`: rotate top three values.
- `+/-`: toggle scalar entry sign.
- `add`: `x + y`.
- `sub`: `x - y`.
- `mul`: `x * y`.
- `div`: `x / y`.
- `enter`: parse scalar entry and push; if entry is empty, duplicate top value.
- `0..9`: append digit to scalar entry.
- `decimal`: append decimal point to scalar entry.
- `.`: append decimal point to scalar entry.
- `exp`: insert scientific exponent marker `E` in scalar entry.
- `undo`: restore state before the last successful operation.

## Matrix Builder Panel

- `apply`: rebuild Matrix Entry text using current row/column size controls.
- `apply_size`: rebuild Matrix Entry text using current row/column size controls.
- `push`: parse Matrix Entry and push matrix.
- `push_matrix`: parse Matrix Entry and push matrix.
- `preset`: load matrix preset into Matrix Entry.
- `preset_matrix`: load matrix preset into Matrix Entry.
- `preset i(n)`: load identity matrix of size `n` into Matrix Entry.
- `push_identity`: load identity matrix of size `n` into Matrix Entry.
- `import csv -> a`: load CSV file into Matrix Entry.
- `import_csv`: load CSV file into Matrix Entry.
- `export top csv`: export top-of-stack matrix as CSV.
- `export_csv`: export top-of-stack matrix as CSV.
- `hstack`: top value is integer count `n`; consume `n` values and concatenate horizontally.
- `vstack`: top value is integer count `n`; consume `n` values and concatenate vertically.
- `hravel`: matrix -> split into columns on stack; vector -> push elements to stack.
- `vravel`: matrix -> split into rows on stack; vector -> push elements to stack.

## Vector Operators Panel

- `dot`: dot product of two vectors (supports `Nx1`/`1xN`).
  - `arg order`: `x·y` from vectors `x` then `y`.
- `cross`: cross product of two length-3 vectors.
  - `arg order`: `x×y` from vectors `x` then `y`.
- `diag`: vector -> diagonal matrix.
- `tpltz`: vector -> Toeplitz matrix.
- `toep`: vector -> Toeplitz matrix.

## Matrix Operators Panel

- `solve a*x=b`: solve linear system using top two matrices.
  - `arg order`: `A` then `B` on stack computes `A * x = B`.
- `solve_ax_b`: solve linear system using top two matrices (`A`, `B`).
- `lstsq solve`: least-squares solve with pseudoinverse.
  - `arg order`: `A` then `B`; computes `x = A^+ * B`.
- `solve_lstsq`: least-squares solve with pseudoinverse.
- `det`: determinant of top matrix.
- `determinant`: determinant of top matrix.
- `trace`: trace of top matrix.
- `transpose`: transpose of top matrix.
- `inverse`: inverse of top matrix.
- `h mul`: Hadamard element-wise multiply.
  - `arg order`: `x .* y`.
- `hadamard_mul`: Hadamard element-wise multiply.
- `h div`: Hadamard element-wise divide.
  - `arg order`: `x ./ y`.
- `hadamard_div`: Hadamard element-wise divide.
- `norm_p`: matrix/vector p-norm.
- `norm-p`: matrix/vector p-norm.
  - `arg order`: value `x` then scalar `p` computes `||x||_p`.
- `exp(mat)`: matrix exponential `e^A`.
- `mat_exp`: matrix exponential `e^A`.
- `hermitian`: conjugate transpose `A^H`.
- `mat x^y`: integer matrix power.
  - `arg order`: matrix `A` then integer exponent `n` computes `A^n`.
- `mat_pow`: integer matrix power.

## Matrix Decompositions Panel

- `qr`: QR decomposition; returns `Q`, `R`.
- `lu`: LU decomposition with pivot; returns `P`, `L`, `U`.
- `svd`: singular value decomposition; returns `U`, `S`, `Vt`.
- `evd`: eigendecomposition (or Schur fallback with warning).

## Scalar Panel

- `neg`: unary negation.
- `inv`: reciprocal `1/x`.
- `square`: square `x^2`.
- `sqrt`: square root.
- `pow`: power.
  - `arg order`: `x^y`.
- `root`: y-th root.
  - `arg order`: `x^(1/y)`.
- `exp10`: `10^x`.
- `log10`: `log10(x)`.
- `exp`: `e^x`.
- `ln`: natural logarithm.
- `exp2`: `2^x`.
- `log2`: `log2(x)`.
- `log_y_x`: logarithm with arbitrary base.
  - `arg order`: base `x`, value `y`, computes `log_x(y)`.
- `signum`: complex/real signum.
- `percent`: percent-of operation.
  - `arg order`: base `x`, percent `y`, computes `x*y/100`.

## Trigonometry Panel

- `hyp`: toggle trig buttons between circular and hyperbolic variants.
- `hyp_toggle`: toggle trig buttons between circular and hyperbolic variants.
- `to rad`: convert degrees to radians.
- `to_rad`: convert degrees to radians.
- `to deg`: convert radians to degrees.
- `to_deg`: convert radians to degrees.
- `atan2`: two-argument arctangent.
  - `arg order`: `atan2(y, x)` where stack has `x` then `y`.
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
- `sinh`: hyperbolic sine.
- `asinh`: inverse hyperbolic sine.
- `cosh`: hyperbolic cosine.
- `acosh`: inverse hyperbolic cosine.
- `tanh`: hyperbolic tangent.
- `atanh`: inverse hyperbolic tangent.
- `sech`: hyperbolic secant.
- `asech`: inverse hyperbolic secant.
- `csch`: hyperbolic cosecant.
- `acsch`: inverse hyperbolic cosecant.
- `coth`: hyperbolic cotangent.
- `acoth`: inverse hyperbolic cotangent.

## Complex Panel

- `abs`: magnitude.
- `abs^2`: squared magnitude.
- `abs_sq`: squared magnitude.
- `arg`: phase angle.
- `conjugate`: complex conjugate.
- `real()`: real part.
- `real_part`: real part.
- `imag()`: imaginary part.
- `imag_part`: imaginary part.
- `cart`: rectangular compose/decompose.
  - `arg order`: if composing, `a` then `b` -> `a + bi`.
- `pol`: polar compose/decompose.
  - `arg order`: if composing, magnitude `r` then angle `theta`.
- `npol`: normalized polar compose/decompose.
  - `arg order`: if composing, magnitude `r` then cycles `c` where angle = `2*pi*c`.

## Special Panel

- `factorial`: factorial `n!` (integer domain).
- `ncr`: combinations.
  - `arg order`: `n` then `r`, computes `nCr`.
- `npr`: permutations.
  - `arg order`: `n` then `r`, computes `nPr`.
- `x mod y`: Euclidean modulo.
  - `arg order`: `x mod y`.
- `modulo`: Euclidean modulo (`x mod y`).
- `gcd`: greatest common divisor.
  - `arg order`: `gcd(x, y)`.
- `lcm`: least common multiple.
  - `arg order`: `lcm(x, y)`.
- `gamma`: gamma function `Gamma(x)`.
- `erf`: error function.
- `erfc`: complementary error function.
- `bessel`: Bessel function of the first kind (`J0`).
- `mbessel`: modified Bessel function (`I0`).
- `sinc`: normalized sinc `sin(x)/x` with limit value at zero.

## Statistics Panel

- `mean`: arithmetic mean.
- `mode`: statistical mode.
- `median`: median.
- `quart`: quartile summary `[min, q1, q2, q3, max]`.
- `std dev p`: population standard deviation.
- `std_dev_p`: population standard deviation.
- `std dev s`: sample standard deviation.
- `std_dev_s`: sample standard deviation.
- `variance`: population variance.
- `max`: maximum.
- `min`: minimum.
- `ran#`: pseudo-random scalar in `[0, 1)`.
- `rand_num`: pseudo-random scalar in `[0, 1)`.

## Rounding Panel

- `round`: round to nearest integer.
- `floor`: floor.
- `ceil`: ceiling.
- `dec part`: decimal part (`x - trunc(x)`).
- `dec_part`: decimal part (`x - trunc(x)`).

## Constants Panel

- `pi`: push `pi`.
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

- `sto`: store top stack value in selected register.
- `memory_store`: store top stack value in selected register.
- `rcl`: recall selected register to stack top.
- `memory_recall`: recall selected register to stack top.
- `clr`: clear selected register.
- `memory_clear`: clear selected register.

## Keyboard Shortcuts

- `delete`: drop.
- `d`: dup.
- `s`: swap.
- `u`: undo.
- `r`: begin roll command entry.
- `p`: begin pick command entry.
- `esc`: cancel pending roll/pick or close help dialog.
