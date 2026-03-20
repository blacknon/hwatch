# hwatch plugin cdylib sample

This example is a draft plugin for a future `hwatch` diffmode plugin system.

It does not depend on internal `hwatch` crates on purpose. The ABI boundary is
kept in plain C-compatible structs and UTF-8 bytes, so the host can load it with
`libloading` and convert the returned JSON into its own internal types.

## Build

```bash
cargo build --manifest-path examples/plugin_cdylib_sample/Cargo.toml --release
```

The resulting dynamic library will be created under:

- `examples/plugin_cdylib_sample/target/release/libhwatch_plugin_cdylib_sample.dylib`
- or the platform equivalent such as `.so` / `.dll`

## Exported symbols

- `hwatch_diffmode_metadata`
- `hwatch_diffmode_generate`
- `hwatch_diffmode_free_bytes`

## Output format

`hwatch_diffmode_generate` returns a UTF-8 JSON document like:

```json
{
  "schema_version": 1,
  "header_text": "LineNum",
  "lines": [
    "  1 |    count: 10",
    "    | ^  numeric delta: 10 -> 15 (+5)",
    "  2 | -  count: 10",
    "  2 | +  count: 15"
  ]
}
```

## Behavior

The sample mode is line-diff based.

- Equal lines are rendered like a normal line diff.
- Replaced lines are rendered as `-` and `+`.
- If a replaced line only changes numeric values with the same surrounding text,
  the plugin inserts extra helper line(s) above the change and shows each
  numeric delta on its own line.
- `--diff-output-only` is supported.

The plugin name is `line-num-diff`.
