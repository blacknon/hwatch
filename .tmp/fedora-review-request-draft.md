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
- GitHub Actions validates RPM packaging flow in a Fedora container
- Strict RPM packaging CI also succeeds
- Upstream test suite passes during packaging validation

Potential reviewer notes:

- The application can load optional native diff plugins at runtime
- Review may decide whether future package split is useful
- Upstream is willing to refine the spec further based on Fedora review feedback
