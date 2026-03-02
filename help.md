# Calculator Help

This calculator uses Reverse Polish Notation (RPN):

1. Type a value into the entry line.
2. Press `Enter` to push it onto the stack.
3. Apply operators to the top stack value(s).

Stack labels:

- `T`: top of stack
- `#n`: lower stack positions

## Scalar Keypad

- `RAD`: set angle mode to radians
- `DEG`: set angle mode to degrees
- `CE`: clear current entry buffer
- `C`: clear stack + entry
- `Drop`: remove top stack value
- `Dup`: duplicate top stack value
- `Swap`: swap top two stack values
- `Rot`: rotate top three values
- `+/-`: toggle sign on current entry
- `/`, `*`, `-`, `+`: arithmetic
- `0..9`, `.`: number entry
- `Enter`: push entry buffer onto stack
- `EXP`: compute `10^x` on top stack value

## Matrix Panel

- `Push A`, `Push B`: parse matrix text and push to stack
- `Size A` / `Size B` controls:
  - set rows + columns
  - press `Apply` to regenerate matrix text with that size
- `Solve A*x=B`: solve linear system using top two matrices
- `det`: determinant of top matrix
- `transpose`: transpose top matrix
- `inverse`: inverse of top matrix
- `Preset A 2x2`, `Preset B vec`: convenience templates
- `Push I(n)`: push identity matrix of size `n`

Matrix input format:

- One row per line
- Values separated by spaces or commas

Example:

```text
1 2
3 4
```

## Scientific Panel

### Trig / Angle

- `sin`, `asin`, `cos`, `acos`, `tan`, `atan`
- `atan2`: binary operation using stack order `(y, x)`
- `to_rad`: convert top real value degrees -> radians
- `to_deg`: convert top real value radians -> degrees
- `hyp`: toggles trig buttons to hyperbolic/inverse-hyperbolic variants:
  - `sinh`, `asinh`, `cosh`, `acosh`, `tanh`, `atanh`

### Powers / Logs / Core

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

- `abs`: magnitude
- `abs^2`: squared magnitude
- `arg`: phase/argument (respects RAD/DEG mode)
- `conj`: complex conjugate
- `Push a+bi`: push complex from real/imag input fields

### Integer / Number Tools

- `n!`: factorial (non-negative integer)
- `nCr`: combinations
- `nPr`: permutations
- `x mod y`: Euclidean remainder
- `ran#`: push pseudo-random number in `[0, 1)`
- `GCD`: greatest common divisor (integers)
- `LCM`: least common multiple (integers)
- `rnd`: round to nearest integer
- `floor`: floor
- `ceil`: ceil
- `decP`: decimal part (`x - trunc(x)`)

## Memory (A-Z)

- Choose register letter in input (`A` to `Z`)
- `STO`: store top stack value
- `RCL`: recall register onto stack
- `CLR`: clear register

## Keyboard Shortcuts

- `0-9`: append digit to entry
- `.`: decimal point
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
