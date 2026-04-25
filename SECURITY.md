# Security Policy

Thank you for helping keep `hwatch` and its bundled plugins safe.

## Supported Versions

Security fixes are best-effort for:

- the latest released version
- the current default branch

Older releases may not receive security updates.

## Reporting a Vulnerability

Please do not open a public GitHub issue for suspected security vulnerabilities.

Instead, report them privately to the maintainer:

- `blacknon@orebibou.com`

If GitHub private vulnerability reporting is enabled for the repository, you may use that as well.

## What to Include

Please include as much of the following as possible:

- affected version or commit
- operating system and terminal environment
- steps to reproduce
- proof-of-concept input, command, plugin, or file
- expected behavior
- actual behavior
- impact assessment
- whether the issue requires a specific terminal, shell, plugin, or environment setting

If the report involves secrets, credentials, private files, or local environment details, please redact anything unnecessary before sending it.

## Scope

Examples of security-relevant issues include:

- command execution bugs that bypass expected user intent
- unsafe plugin loading or ABI handling issues
- path handling issues affecting local files
- untrusted input leading to crashes, memory safety concerns, or privilege boundary problems
- packaging or release issues that could affect distributed artifacts

General usage questions, feature requests, and non-security bugs should go through the normal issue tracker.

## Disclosure Process

After receiving a report, the maintainer will try to:

- confirm whether the issue is reproducible
- assess severity and affected versions
- prepare a fix or mitigation
- coordinate disclosure after a fix is available when possible

Response time is best-effort, so please allow some time for investigation.

## Coordinated Disclosure

Please avoid public disclosure until the issue has been reviewed and, when possible, fixed.

If you believe the issue is especially severe, mention that clearly in the report so it can be prioritized.
