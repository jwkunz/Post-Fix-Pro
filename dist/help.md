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
- `0..9`, `.`, `EXP`: number entry (`EXP` inserts `E` for scientific notation, e.g. `2.3E3`)
- `Enter`: push entry buffer onto stack

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
- Values are parsed when you press `Push A` / `Push B` (free-form text allowed while typing)
- Scientific notation is supported in values (e.g. `-1.2E-3`)
- Keyboard focus in matrix/complex text fields captures typing (calculator hotkeys are suppressed there)
- Use `Shift+Enter` for explicit newline entry while editing matrices

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
- Complex entry mode selector:
  - `a + bi`: enter real and imaginary parts directly
  - `mag + arg`: enter magnitude and argument, converted using current RAD/DEG mode
  - `a * exp(i 2 pi b)`: enter scale `a` and cycle phase `b` (`theta = 2πb`)
- Complex fields accept free-form text while typing; parsing/validation happens when pressing `Push ...`
- Scientific notation is supported in complex fields (e.g. `2.5E2`)
- `Push ...`: pushes the converted complex value onto the stack

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
