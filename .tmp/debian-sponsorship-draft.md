# Debian Sponsorship Request Draft

Subject: RFS: hwatch/0.4.2-1 -- modern alternative to the watch command with history and diff views

Dear mentors,

I am looking for a sponsor for my package `hwatch`.

Package name    : hwatch
Version         : 0.4.2-1
Upstream Author : blacknon <blacknon@orebibou.com>
URL             : https://github.com/blacknon/hwatch
License         : MIT
Section         : utils

Short description:

 hwatch is an interactive terminal application similar to watch.
 It records command output over time, allows users to browse history,
 inspect differences between runs, export logs, and optionally trigger
 follow-up commands when output changes.

Current preparation status:

- Debian packaging metadata has been added upstream
- Debian binary package builds successfully in CI in a `debian:sid` container
- Upstream tests pass with the current dependency constraints
- Fedora/RPM packaging also builds successfully in CI
- The package ships man page and shell completion files

Remaining caveats:

- The current upstream packaging flow is suitable for CI validation, but
  Debian Rust Team review may still request dependency or policy adjustments
- Runtime plugin loading exists as an optional feature and may need packaging
  discussion if reviewers prefer a split or stricter framing

Repository:

- https://github.com/blacknon/hwatch

Additional notes for reviewers:

- This package is written in Rust
- Debian and Fedora packaging checks are both tracked in GitHub Actions
- Upstream includes man page and shell completion files

Thank you for your time and review.
