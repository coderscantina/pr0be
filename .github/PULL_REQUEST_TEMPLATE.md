## Summary

<!-- What does this PR change, and why? Describe the intent, not just the diff. -->

## Linked issue

<!-- Closes #123 / Relates to #456 — or "none" for small standalone fixes. -->

## How was this tested?

<!-- Which repository states did you build to exercise this? Paste the git commands
     that create the fixture if a reviewer should reproduce it. -->

## AI assistance

<!-- If any part of this contribution was meaningfully shaped by an AI tool, say so.
     The spirit of the rule: if you'd credit a human pair-programmer, credit the AI.
     Example:
     > AI assistance: Used Claude to draft the worktree detection, then reviewed and adjusted manually.
     Autocomplete suggestions and minor one-liners don't need disclosure. -->

## Checklist

- [ ] Commits follow the [Gitmoji format](CONTRIBUTING.md#commit-messages--gitmoji-required)
- [ ] `cargo fmt --all` and `cargo clippy --all-targets` are clean
- [ ] `cargo test` passes locally (and new behavior is covered where it makes sense)
- [ ] `README.md` / `CHANGELOG.md` updated if behavior changed
- [ ] The change keeps pr0be read-only — it does not write to scanned repositories
- [ ] AI usage is disclosed above (or not applicable)
- [ ] I understand and can explain the code I'm submitting
