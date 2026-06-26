# Nous Lang Examples

These examples are intended for users of the packaged `nlang` tool.

- `valid/`: programs that should pass `nlang check` and `nlang run`.
- `invalid/`: small programs that intentionally fail so diagnostic output can be inspected.

From the repository root:

```powershell
cargo run -p nous_cli -- run examples/valid/calculator.nl
cargo run -p nous_cli -- check examples/invalid/type_mismatch.nl
```

From the portable package root:

```powershell
.\bin\nlang.exe run .\examples\valid\calculator.nl
.\bin\nlang.exe check .\examples\invalid\type_mismatch.nl
```
