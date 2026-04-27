# Fedora Review Checklist

## Current status

- `hwatch.spec` exists and builds in CI in a Fedora container
- RPM packaging validation runs in GitHub Actions
- Strict RPM packaging validation also passes in CI
- Upstream project ships man page and shell completions

## Before filing a Fedora review request

1. Check whether Fedora prefers using Rust packaging macros for this package.
2. Verify completion install paths and man page compression handling.
3. Confirm license metadata and `%license` usage are acceptable.
4. Verify source tarball layout matches what the spec expects.
5. Review whether bundled example plugins should stay out of the main package.
6. Prepare SRPM generation and submission steps for the review ticket.

## Likely reviewer questions

- Why this tool should be packaged instead of relying on `cargo install`
- Whether plugin loading introduces any special security or packaging concerns
- Whether all runtime assets are installed in standard Fedora locations
- Whether the package should be split in the future

## Helpful links

- https://docs.fedoraproject.org/en-US/packaging-guidelines/
- https://docs.fedoraproject.org/en-US/package-maintainers/Package_Review_Process/
