# Changelog

All notable changes to pr0be are documented here.
Commits follow the [Gitmoji](https://gitmoji.dev/) convention.

## [Unreleased]

## [v0.1.0] — 2026-07-31

First sweep.

- 🎉 Scan configurable directory trees for git repositories, including repositories nested inside other repositories
- ✨ Report uncommitted work: staged, modified, and untracked files
- ✨ Report code that was never pushed: branches ahead of upstream, branches without an upstream, repositories without a remote, and vanished upstreams
- ✨ Report stash entries and detached HEADs — both hold work that lives only in this working copy
- ✨ Flag project directories (package.json, Cargo.toml, composer.json, …) that are not under git control at all
- ✨ Optional `--fetch` for accurate behind-remote detection, with terminal prompts disabled and a per-repository timeout
- ✨ Native desktop notification via `--notify`, fired only when something is at risk
- ✨ TOML configuration with roots, ignore list, project markers, and depth limit; CLI flags override it
- ✨ `--json` output and a non-zero exit code for cron, hooks, and CI
- ⚡️ Parallel repository inspection — ~24 000 directories in about a second
- 📝 README, contributing guide, code of conduct, and security policy
