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
- Packaging-oriented CI runs in a Debian container
- Upstream tests pass with the current dependency constraints
- Some dependency alignment and policy review may still need adjustment
  before upload, especially around Rust crate availability in Debian

Repository:

- https://github.com/blacknon/hwatch

Additional notes for reviewers:

- This package is written in Rust
- The project also carries RPM packaging metadata and CI validation
- Upstream includes man page and shell completion files

Thank you for your time and review.
