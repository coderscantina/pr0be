# Contributing to pr0be

Thank you for being here. Whether you're fixing a typo, reporting a false positive, or adding a whole new class of check, every contribution makes `pr0be` better at its one job: making sure no code is left stranded on a single machine.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Getting Started](#getting-started)
- [Commit Messages — Gitmoji Required](#commit-messages--gitmoji-required)
- [AI-Assisted Contributions](#ai-assisted-contributions)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)

---

## Code of Conduct

We expect everyone in this community to be kind, patient, and constructive. Harassment, gatekeeping, and dismissiveness have no place here. Treat people the way you'd want to be treated on a bad day. The full text lives in [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

Violations can be reported to [hello@coderscantina.com](mailto:hello@coderscantina.com).

---

## Ways to Contribute

You don't have to write Rust to make a difference:

- **Report a false positive or false negative** — a repository state `pr0be` gets wrong is the most valuable bug report this project can receive
- **Improve the docs** — spotted something confusing? Fix it
- **Add project markers** — know an ecosystem whose manifest file we don't detect yet? Add it
- **Review pull requests** — fresh eyes catch what the author can't
- **Write code** — new checks, performance work, platform support, tests

---

## Getting Started

```sh
git clone https://github.com/coderscantina/pr0be.git
cd pr0be
cargo build
cargo test
./target/debug/pr0be ~/some/tree
```

You need Rust 1.74+ and `git` on your `PATH`.

The code is deliberately small — five modules, each with one responsibility:

| file            | responsibility                                              |
| --------------- | ----------------------------------------------------------- |
| `src/main.rs`   | CLI parsing, wiring, exit codes                              |
| `src/config.rs` | config file loading, defaults, `--init` template             |
| `src/scan.rs`   | the directory walk: what is a repository, what is a project  |
| `src/git.rs`    | running git, parsing its output into issues                  |
| `src/report.rs` | human-readable table, summary line, notification text        |
| `src/notify.rs` | the platform notifier                                        |

Adding a check usually means: a variant in `git::Issue`, a case in `label()`/`detail()`/`color_for()`, and the detection itself in `git::inspect`.

### Testing against real repositories

Please don't test only against your own tree — build fixtures. A shell script that creates a bare "remote", a clone that is ahead, one with a stash, and a folder with a `package.json` and no `.git` will exercise most of the logic in under a second.

---

## Commit Messages — Gitmoji Required

Commits start with a [Gitmoji](https://gitmoji.dev/), followed by an imperative summary:

```
✨ Report worktrees whose branch has no upstream
🐛 Stop counting nested repositories as untracked files
📝 Document the JSON output shape
⚡️ Inspect repositories in parallel
♻️ Split the walker out of main
✅ Add fixtures for the behind/ahead parser
```

Common ones here: ✨ feature · 🐛 fix · 📝 docs · ♻️ refactor · ⚡️ performance · ✅ tests · 👷 CI · 🔒 security.

---

## AI-Assisted Contributions

AI tools are welcome — this project was itself drafted with one. Two rules:

1. **Disclose meaningful assistance.** If you'd credit a human pair-programmer, credit the AI in your PR description. Autocomplete and one-liners don't need a mention.
2. **Understand what you submit.** You must be able to explain every line in your PR and defend the design choices in review. Unreviewed generated code wastes maintainer time and will be closed.

---

## Pull Request Process

1. Fork and branch from `main` (`fix/nested-untracked`, `feat/worktree-check`).
2. Keep the PR focused — one concern per pull request.
3. Make sure `cargo test`, `cargo clippy --all-targets`, and `cargo fmt --check` pass.
4. Update `README.md` and `CHANGELOG.md` under `[Unreleased]` when behavior changes.
5. Fill in the PR template, including how you tested it.
6. Expect review comments — they're about the code, never about you.

---

## Code Style

- `cargo fmt` is the formatter; `cargo clippy --all-targets` must be warning-free.
- No `unwrap()` on anything that depends on the user's filesystem or on git's output — a hostile directory tree must never panic the sweep.
- `pr0be` is read-only by contract. A change that writes to a scanned repository, or that makes network calls outside the opt-in `--fetch`, will not be merged.
- Prefer plain `std` over a new dependency. Every crate added is a crate someone has to trust.
- Comments explain *why*, not *what*.

---

## Reporting Bugs

Open a [bug report](https://github.com/coderscantina/pr0be/issues/new?template=bug_report.yml). The single most useful thing you can include is a minimal reproduction: the sequence of `git` commands that produces the repository state `pr0be` gets wrong.

---

## Suggesting Features

Open a [feature request](https://github.com/coderscantina/pr0be/issues/new?template=feature_request.yml). Good candidates are new ways code can be stranded on one machine. Things that are explicitly out of scope: fixing problems automatically (committing, pushing, `git init`), and anything that mutates a scanned repository.
