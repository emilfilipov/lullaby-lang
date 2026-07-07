# Lullaby Examples

These examples are intended for users of the packaged `lullaby` tool.

- `valid/`: programs that should pass `lullaby check` and `lullaby run`.
- `invalid/`: small programs that intentionally fail so diagnostic output can be inspected.

From the repository root:

```powershell
cargo run -p lullaby_cli -- run examples/valid/calculator.lby
cargo run -p lullaby_cli -- check examples/invalid/type_mismatch.lby
```

From the portable package root:

```powershell
.\bin\lullaby.exe run .\examples\valid\calculator.lby
.\bin\lullaby.exe check .\examples\invalid\type_mismatch.lby
```

## Selected examples

- `valid/primes.lby`: counts the prime numbers below 50 with trial division
  (defines `rem` and `is_prime` helpers, a `while` loop, and early `return`).
- `valid/collatz.lby`: prints Collatz stopping times for a few integers, using
  only integer arithmetic and an even/odd `rem` check.
- `invalid/int_float_mismatch.lby`: mixes `i64` and `f64` in one expression
  (`let x i64 = 1 + 2.0`); `lullaby check` reports diagnostic `L0307` (operands
  of `+` must share a type) and exits non-zero.
