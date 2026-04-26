# Fedora Review Request Draft

Package Review Request
======================

SRPM Name or Package Name:
- hwatch

Short description:
- Modern alternative to the `watch` command with history and diff views

Project URL:
- https://github.com/blacknon/hwatch

Upstream license:
- MIT

Why this package should exist in Fedora:

- It provides a richer workflow than classic `watch`
- Users can inspect output history and multiple diff modes directly in the terminal
- The project already ships man pages and shell completion files
- RPM packaging metadata exists upstream and is validated in CI in a Fedora container

Packaging status:

- `hwatch.spec` exists upstream
- GitHub Actions validates RPM packaging flow
- Current spec still prioritizes upstream portability and CI validation;
  further Fedora-specific review feedback can be incorporated as needed

Potential reviewer notes:

- The application can load optional native diff plugins at runtime
- Review may decide whether future package split is useful
- The current CI path uses `--nodeps`; final Fedora submission should be
  validated in a proper Fedora packaging environment as well
