# hwatch plugin numeric inline diff

This bundled plugin keeps a line-diff style layout and highlights changed
numeric tokens inline.

It is aimed at outputs like `df`, `du`, counters, metrics, and tabular command
output where "the line changed" is useful, but "which number increased or
decreased" is the most important signal.

## Build

```bash
cargo build --manifest-path plugins/numeric-inline-diff/Cargo.toml --release
```

The resulting dynamic library will be created under:

- `plugins/numeric-inline-diff/target/release/libhwatch_plugin_numeric_inline_diff.dylib`
- or the platform equivalent such as `.so` / `.dll`

## Behavior

- Equal lines are rendered like a normal line diff.
- Changed lines are rendered as `-` and `+`.
- If a removed line and an added line have the same non-numeric structure, the
  plugin pairs them and highlights changed numeric tokens inline.
- Increased values are colored green.
- Decreased values are colored red.
- `--diff-output-only` is supported.

The plugin name is `numeric-inline-diff`.
