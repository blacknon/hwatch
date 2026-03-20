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
  "header_text": "Summary",
  "lines": [
    "  1 |   same text",
    "  2 | ~ before -> after"
  ]
}
```

The sample mode is intentionally simple: it emits a summary-oriented diff rather
than line, word, or watch highlighting.

