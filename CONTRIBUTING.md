# Contributing

Thanks for your interest in contributing to `hwatch`.

This document explains the expected workflow for code, documentation, plugins, and release-related changes so contributors can make focused pull requests that are easy to review.

## Before You Start

- Check existing issues and pull requests before starting large changes.
- Prefer small, focused pull requests over broad refactors.
- If your change affects behavior, CLI output, docs, or packaged artifacts, update the related files in the same pull request.

## Development Setup

### Requirements

- Rust stable
- `cargo`
- On macOS, Linux, or another environment supported by the project

Optional tools:

- `mise` for project tasks
- `vhs` for regenerating demo GIFs in `img/`

### Build

Build the main binary:

```bash
cargo build
```

Build everything in the workspace:

```bash
cargo build --workspace --all-targets
```

Build bundled plugins:

```bash
cargo build --manifest-path plugins/numeric-diff/Cargo.toml --release
cargo build --manifest-path plugins/numeric-inline-diff/Cargo.toml --release
```

If you use `mise`, these tasks are also available:

```bash
mise run cli_build
mise run plugin_build
mise run full_build
```

## Checks Before Opening a PR

Please run the same core checks used in CI:

```bash
cargo fmt --all --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

If your change only affects a plugin, run the relevant plugin build too.

## Code Style

- Follow standard Rust formatting with `cargo fmt`.
- Keep changes focused and avoid unrelated cleanup in the same pull request.
- Add tests for bug fixes or behavior changes when practical.
- Prefer keeping CLI help text, README examples, and manual pages consistent.

## Documentation Expectations

When behavior changes, update the related documentation in the same pull request.

Common files to review:

- `README.md` for user-facing behavior and examples
- `man/man.md` and generated manpage sources when CLI text changes
- `completion/` when command-line options or help text change
- `img/*.tape` and generated GIFs when demos become outdated
- `hwatch.spec` when packaging or release build behavior changes
- `plugins/*/README.md` when bundled plugin behavior changes

## Tests and Fixtures

- Unit and integration tests live in the Rust workspace and `tests/`.
- Some features are easier to validate manually because `hwatch` is a TUI tool.
- If you verify something manually, include the steps in the pull request description.

Examples:

- terminal behavior
- mouse support
- PTY-related behavior
- diff mode rendering
- plugin loading

## Pull Requests

Please include:

- a short summary of what changed
- why the change was needed
- how you tested it
- any follow-up work or known limitations

Good pull requests usually:

- keep one main purpose
- include docs updates when needed
- avoid mixing refactors with behavior changes unless necessary

## Reporting Bugs

When opening an issue, include as much of the following as possible:

- operating system
- terminal emulator
- `hwatch` version
- exact command used
- expected behavior
- actual behavior
- reproduction steps

For rendering or terminal issues, screenshots or terminal recordings can help.

## Plugin Contributions

`hwatch` supports diffmode plugins as dynamic libraries.

If you change plugin APIs or bundled plugins:

- update the relevant plugin README
- verify the plugin still builds in release mode
- document any compatibility or ABI considerations in the pull request

## Release-Related Changes

If your change affects packaging or distributed artifacts, also review:

- `.github/workflows/`
- `hwatch.spec`
- `package/`
- release packaging paths or asset names

## Questions

If you are unsure about scope or direction, opening an issue or draft pull request early is welcome.
