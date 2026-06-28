# Lullaby Examples

These examples are intended for users of the packaged `lullaby` tool.

- `valid/`: programs that should pass `lullaby check` and `lullaby run`.
- `invalid/`: small programs that intentionally fail so diagnostic output can be inspected.

From the repository root:

```powershell
cargo run -p lullaby_cli -- run examples/valid/calculator.lullaby
cargo run -p lullaby_cli -- check examples/invalid/type_mismatch.lullaby
```

From the portable package root:

```powershell
.\bin\lullaby.exe run .\examples\valid\calculator.lullaby
.\bin\lullaby.exe check .\examples\invalid\type_mismatch.lullaby
```
