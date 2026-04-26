# Debian Packaging Notes

This file keeps temporary packaging notes outside the committed Debian
packaging directory.

Before attempting an official Debian upload, verify at least the following:

- Resolve direct and transitive Rust `Build-Depends` as Debian packages
  (typically `librust-*-dev` via `dh-cargo` and the Debian Rust Team workflow).
- Confirm that the package builds without network access.
- Run `lintian` and a clean chroot build such as `sbuild` or `pbuilder`.
- Review whether plugin support should stay enabled by default for Debian.

Current dependency notes from Debian package index checks:

- `librust-crossbeam-channel-dev` is present in sid at `0.5.15`.
- `librust-chardetng-dev` is present in sid at `0.1.17`.
- `librust-termwiz-dev` is present in sid at `0.23.3`.
- `librust-libloading-dev` is present in sid, but the packaged version seen was
  `0.8.5` while `Cargo.toml` currently asks for `0.8.9`.
- `librust-crossterm-dev` is present in sid, but the packaged version seen was
  `0.28.1` while `Cargo.toml` currently asks for `0.29.0`.
- `librust-ratatui-dev` is present in sid, but the packaged version seen was
  `0.28.x`/`0.29.0` while `Cargo.toml` currently asks for `0.30.0`.
- `librust-config-dev` is present in sid as source package `rust-config` at
  `0.15.9`.
- `librust-nix-dev` is present in sid at `0.30.1`.
- `librust-serde-dev` is present in sid at `1.0.228`.
- `librust-shell-words-dev` is present in sid at `1.1.0`.
- `librust-ctrlc-dev` was observed in Debian bookworm at `3.2.3`; Debian sid
  should be rechecked during an actual package build if feature metapackages are
  missing.

This means Debian packaging work will likely require either:

- relaxing some crate version requirements, or
- updating the relevant crates in Debian first, or
- carrying a Debian-specific patch set if the code still works with the Debian
  crate versions.

Repository follow-up applied:

- `Cargo.toml` was relaxed to allow:
  - `ratatui >=0.29, <0.31`
  - `crossterm >=0.28.1, <0.30`
  - `libloading >=0.8.5, <0.9`
  - `chrono >=0.4.42, <0.5`
  - `config >=0.15.9, <0.16`
  - `nix >=0.30.1, <0.32`
  - `ctrlc >=3.2.3, <4`
  - `shell-words >=1.1.0, <2`

This should reduce version skew for Debian packaging without forcing the
workspace lockfile away from the currently tested dependency set.

Validation status:

- `CARGO_TARGET_DIR=/tmp/hwatch-target cargo check --locked` passed locally.
- `CARGO_TARGET_DIR=/tmp/hwatch-target cargo test --locked --all-features -- \
  --skip test_exec_command_with_force_color_stdout_is_tty \
  --skip test_exec_command_with_force_color_stdin_is_tty` passed locally.
- Current result: the relaxed dependency constraints remain compatible with the
  current codebase and lockfile.

Useful references:

- https://mentors.debian.net/
- https://wiki.debian.org/Teams/RustPackaging

## Follow-up Checklist

1. Add the required `librust-*-dev` `Build-Depends` for all direct dependencies in `Cargo.toml`.
2. Verify whether current Debian versions of `ratatui` and related crates satisfy the codebase.
3. Build with `sbuild`/`pbuilder` and fix missing dependencies or policy issues.
4. Run `lintian` and address warnings.
5. Review whether examples/plugins should be excluded or split for Debian.
6. File an ITP and request sponsorship when the package is buildable.
