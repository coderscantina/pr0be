# Security Policy

`pr0be` is a local command-line tool that walks directory trees and runs `git` in them. Its security-sensitive surface is small but real: it executes an external binary, it reads paths and repository output it does not control, and with `--fetch` it makes network calls.

## Supported Versions

Security fixes land on the latest release published from `main`. Older releases are not patched — please stay current.

| Release | Status        |
| ------- | ------------- |
| Latest  | Supported     |
| Older   | Not supported |

## Reporting a Vulnerability

**Do not open a public GitHub issue for an unpatched vulnerability.**

Report privately, in order of preference:

1. **GitHub Security Advisories** — [submit a security advisory](../../security/advisories/new)
2. **Email** — `security@coderscantina.com`, subject line `pr0be security report`

Please include as much of the following as you can:

- `pr0be --version` and how you installed it
- Operating system and `git --version`
- The directory or repository state that triggers the issue
- Reproduction steps or a proof of concept
- Impact summary: what an attacker gains, and what they need to control first

## Response Expectations

- Acknowledgement within **72 hours**.
- Status update within **14 calendar days** if a fix is not yet ready.
- Public disclosure is coordinated after a fix or mitigation exists.
- Reporters are credited in the advisory and release notes, with consent.

## What We Consider a Security Issue

- Command injection or argument injection through a path, branch name, remote URL, or config value
- Escaping the configured roots — following a symlink or path traversal into a tree the user did not ask to scan
- Writing to, or otherwise mutating, a scanned repository; `pr0be` is read-only by contract, and any write is a bug
- Leaking credentials, tokens, or repository contents into output, logs, or notifications
- Executing a binary from a scanned directory rather than from `PATH`
- Unexpected network traffic without `--fetch`, or a `--fetch` that bypasses the terminal-prompt and timeout guards
- Supply chain issues: tampered release artifacts, compromised dependencies

## What Usually Does Not Qualify

- False positives or false negatives in the reported findings — those are ordinary bugs, please file an issue
- A slow scan on a very large tree
- `git` itself prompting for credentials when you explicitly pass `--fetch` in an interactive shell
- Anything requiring the attacker to already have write access to your home directory or your `PATH`

If you're unsure whether something qualifies, report it privately first — we'd rather triage a non-issue than miss a real one.

## Other Platforms

All reports must come through GitHub Security Advisories or the email address above. Direct messages to maintainers on other channels are not a supported reporting path.
