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

## Scripting Guide

The scripting interface runs calculator commands against the same Rust backend used by the main UI. A script is an RPN command stream: push values first, then apply operators.

### How scripts are read

- Scripts run left-to-right, top-to-bottom.
- Blank lines are ignored.
- Comments start with `#` or `//`.
- Tokens are separated by whitespace, except inside complex literals `(re, im)` and matrix literals `[ ... ]`.
- A failing command stops the script and reports the line and column where it failed.

Example:

```text
# Compute (2 + 3) * 4
2 3 add
4 mul
```

### Real number literals

Scripts can push real values directly by writing the number token:

- integers: `0`, `7`, `-12`
- decimals: `3.14`, `-0.25`, `0.5`
- scientific notation: `6.02e23`, `1e-9`, `-2.5E4`

Examples:

```text
2 3 add
10 4 sub
6.25 8 mul
1e3 2 div
```

### Arithmetic and operator tokens

Most common operators can be used either by command name or symbol alias.

Binary arithmetic:

- `add` or `+`
- `sub` or `-`
- `mul`, `*`, or `x`
- `div`, `/`, or `\`

Stack operators:

- `drop`
- `dup`
- `swap`
- `rotate` or `rot`
- `enter`
- `clear_entry`
- `clear_all`
- `undo`

Examples:

```text
2 3 +
10 4 -
6 7 x
22 7 /
5 dup mul
1 2 3 rotate
```

### Named calculator commands

Most ribbon and keypad operations are available by their command names. A few common examples:

- scalar math: `pow`, `sqrt`, `ln`, `exp`, `log10`, `percent`
- trig: `sin`, `cos`, `tan`, `asin`, `atan2`
- special: `factorial`, `ncr`, `npr`, `modulo`, `gcd`, `lcm`
- statistics: `mean`, `median`, `mode`, `variance`, `std_dev_p`, `std_dev_s`
- rounding: `round`, `floor`, `ceil`, `dec_part`
- constants: `pi`, `e`
- complex: `abs`, `arg`, `conjugate`, `real_part`, `imag_part`, `cart`, `pol`, `npol`
- matrix/vector: `determinant`, `transpose`, `inverse`, `dot`, `cross`, `trace`, `qr`, `svd`

Examples:

```text
9 sqrt
2 8 pow
90 angle deg sin
10 factorial
pi 2 div sin
```

### Commands that take an argument token

Some commands consume the next token in the script as an argument instead of reading it from the stack.

- `roll N`
- `pick N`
- `store A`
- `recall A`
- `memclear A`
- `identity N`
- `precision N`
- `display fix|sci|eng`
- `angle deg|rad`
- `entry VALUE`
- `matrix [ ... ]`

Examples:

```text
display sci
precision 8
angle deg
12 store A
clear_all
recall A
roll 3
pick 2
entry -4.25
identity 3
```

### Memory registers as variables

V1 scripting uses the existing memory registers as its variable model. Registers are named `A` through `Z`.

- `store A`: store the top stack value in register `A`
- `recall A`: push the stored value from register `A`
- `memclear A`: clear register `A`

Example:

```text
12 store A
8 store B
recall A recall B add
```

### Complex numbers

Complex literals are written as:

```text
(real, imaginary)
```

Examples:

- `(3,4)` pushes `3 + 4i`
- `(-2.5,0)` pushes a purely real complex value
- `(0,-1)` pushes `-i`

Complex values can be used with the normal arithmetic operators and complex commands:

```text
(3,4) abs
(1,2) (3,-1) add
(2,3) conjugate
5 0.9272952180016122 cart
4 0.5 npol
```

Notes:

- `cart` composes rectangular form from two scalar stack values.
- `pol` composes from radius and angle.
- `npol` composes from radius and turns/cycles.
- Complex literals are also valid inside matrix literals.

### Matrices

Matrix literals are written inside square brackets:

```text
[row1; row2; row3]
```

Rules:

- separate rows with `;`
- separate entries with spaces or commas
- all rows must have the same number of columns
- entries can be real numbers or complex literals

Examples:

```text
[1 2; 3 4]
[1, 2; 3, 4]
[(1,0) (0,1); (0,-1) (1,0)]
```

Matrix examples:

```text
[1 2; 3 4] determinant
[1 2; 3 4] transpose
[1 2; 3 4] inverse
[1 0; 0 1] [2 1; 1 2] add
matrix [1 2; 3 4]
identity 3
```

Vector and matrix workflow examples:

```text
[1; 2; 3] [4; 5; 6] dot
[1; 0; 0] [0; 1; 0] cross
[1 2; 3 4] trace
[1 2; 3 4] qr
```

### Display and angle preferences in scripts

Scripts can change calculator preferences as part of a workflow:

```text
display eng
precision 6
angle deg
30 sin
```

These settings affect the calculator state after the script completes.

### Entry buffer command

`entry VALUE` writes to the scalar entry buffer through the backend API. This is useful when a workflow needs to set up a manual-style entry state before an `enter`.

Example:

```text
entry 6.022e23
enter
```

### Full examples

Basic arithmetic:

```text
2 3 add
4 mul
```

Memory-backed variable flow:

```text
12 store A
clear_all
recall A
8 add
```

Complex workflow:

```text
(3,4) abs
(1,2) (3,-1) add
conjugate
```

Matrix workflow:

```text
[1 2; 3 4] determinant
[1 2; 3 4] inverse
[1; 2; 3] [4; 5; 6] dot
```

Mixed formatting workflow:

```text
display sci
precision 8
6.02214076e23
2 div
```
