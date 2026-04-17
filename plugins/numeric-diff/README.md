# hwatch plugin numeric diff

This is a bundled `hwatch` diffmode plugin for numeric change detection.

It renders a normal line-based diff and inserts helper lines for replaced lines
when the surrounding text is unchanged and only numeric values differ.
The plugin returns structured spans so the plugin itself can choose colors for
markers and numeric deltas without hard-coding that logic into `hwatch`.

## Build

```bash
cargo build --manifest-path plugins/numeric-diff/Cargo.toml --release
```

The resulting dynamic library will be created under:

- `plugins/numeric-diff/target/release/libhwatch_plugin_numeric_diff.dylib`
- or the platform equivalent such as `.so` / `.dll`

## Output format

`hwatch_diffmode_generate` returns a UTF-8 JSON document like:

```json
{
  "schema_version": 2,
  "header_text": "NumDiff",
  "lines": [
    {
      "spans": [
        { "text": "8 | " },
        { "text": "*- ", "style": { "fg": "184,134,11" } },
        { "text": "       ", "style": { "fg": "184,134,11" } },
        { "text": "^-2", "style": { "fg": "magenta" } },
        { "text": "      ", "style": { "fg": "184,134,11" } },
        { "text": "^+8", "style": { "fg": "cyan" } }
      ]
    }
  ]
}
```

## Behavior

- Equal lines are rendered like a normal line diff.
- Replaced lines are rendered as `-` and `+`.
- If a replaced line only changes numeric values with the same surrounding text,
  the plugin inserts `*-` and `*+` helper lines aligned with the numeric values.
- `*-` displays `before - after`.
- `*+` displays `after - before`.
- The plugin sets span colors itself:
  `-` is red, `+` is green, annotation lines use a muted yellow,
  `^+N` is cyan, and `^-N` is magenta.
- `hwatch` still accepts older string-based plugin responses, but this plugin
  uses the structured span format so plugin-specific colors stay in the plugin.
- `--diff-output-only` is supported.

The plugin name is `numeric-diff`.
