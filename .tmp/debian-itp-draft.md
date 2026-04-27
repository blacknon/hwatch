# Debian ITP Draft

Subject: ITP: hwatch -- modern alternative to the watch command with history and diff views

Package: wnpp
Severity: wishlist
Owner: blacknon <blacknon@orebibou.com>

* Package name    : hwatch
  Version         : 0.4.2
  Upstream Contact: blacknon <blacknon@orebibou.com>
* URL             : https://github.com/blacknon/hwatch
* License         : MIT
  Programming Lang: Rust
  Description     : modern alternative to the watch command with history and diff views

 hwatch is an interactive terminal application similar to watch.
 It records command output over time, allows users to browse command history,
 inspect differences between runs, export logs, and optionally trigger follow-up
 commands when output changes.

Main features:

- interactive history browsing for command output
- multiple diff display modes
- optional logfile export and reuse
- shell completion files and man page included upstream

Why this package is useful:

- Provides a richer terminal monitoring workflow than classic `watch`
- Supports history browsing and multiple diff presentation modes
- Has existing packaging metadata and CI checks for Debian and RPM workflows
- Is already distributed through several package ecosystems such as Homebrew,
  MacPorts, AUR, Nix, and Alpine edge/testing

Current packaging status:

- Debian packaging metadata exists in the upstream repository
- Debian packaging is validated in CI in a `debian:sid` container
- Fedora/RPM packaging is also validated in CI
- Upstream test suite passes in CI and in local verification

Repository:

- https://github.com/blacknon/hwatch
