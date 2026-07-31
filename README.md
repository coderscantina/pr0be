# pr0be

<div align="center">

**Every probe droid has one job: find what's hiding out there.** `pr0be` sweeps your disk for code that exists nowhere else — uncommitted changes, unpushed branches, stashes, and project folders that were never put under git at all. One command, a few seconds, and you know exactly what a dead SSD would cost you.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Latest release](https://img.shields.io/github/v/release/coderscantina/pr0be)](https://github.com/coderscantina/pr0be/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/coderscantina/pr0be/ci.yml?branch=main)](https://github.com/coderscantina/pr0be/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.74%2B-orange.svg)](https://www.rust-lang.org)

[Install](#install) · [Usage](#usage) · [Configuration](#configuration) · [Run it daily](#run-it-daily) · [Coder's Cantina](https://www.coderscantina.com)

</div>

## Why

Backups cover files. They do not cover intent. The dangerous state is the one git knows about but no server does: three commits ahead on a branch you forgot, a stash from last March, a prototype folder that never got `git init`. Time Machine will happily restore all of it — right up until the day you're on a different machine.

`pr0be` answers one question, fast: **which of my projects exist only here?**

```console
$ pr0be
● ~/Development/products/sabacc_ui/web
  uncommitted    75 staged, 67 modified, 1 untracked
  stashed        2 stash entry/entries
  unpushed       main is 3 commit(s) ahead
  no upstream    switch-2-b10cks has no upstream

● ~/Development/tools/phpantom_lsp
  uncommitted    2 modified

▲ 106 of 136 projects hold code that only exists here: 13 not a repo, 57 uncommitted, 64 unpushed, 17 no remote, 22 stashed, 10 behind
  (24141 directories scanned in 1047ms)
```

## Install

```sh
cargo install --git https://github.com/coderscantina/pr0be
```

Or from a clone:

```sh
git clone https://github.com/coderscantina/pr0be.git
cd pr0be
cargo install --path .        # → ~/.cargo/bin/pr0be
```

Pre-built macOS and Linux binaries are attached to each [release](https://github.com/coderscantina/pr0be/releases).

Requirements: Rust 1.74+ to build, and `git` on your `PATH` at runtime. macOS and Linux are supported; notifications use the system notifier on both.

## Usage

```sh
pr0be                      # scan the configured roots (default ~/Development)
pr0be ~/Code ~/Sites       # scan these trees instead
pr0be --all                # show clean projects too
pr0be --json               # machine readable, for scripts and dashboards
pr0be --fetch              # fetch remotes first, so "behind" is accurate
pr0be --notify             # desktop notification when something is at risk
pr0be --init > ~/.config/pr0be/config.toml
```

Options: `-c/--config FILE`, `--max-depth N`, `--no-color`, `-h/--help`, `-V/--version`.

### What it reports

| finding         | meaning                                                     |
| --------------- | ----------------------------------------------------------- |
| `not a repo`    | project directory (package.json, Cargo.toml, …) without git  |
| `uncommitted`   | staged / modified / untracked files in the working tree      |
| `unpushed`      | branch is ahead of its upstream                              |
| `no upstream`   | local branch that was never pushed                           |
| `no remote`     | repository has no remote at all — nothing to push to         |
| `upstream gone` | the upstream branch disappeared from the remote              |
| `stashed`       | stash entries — they live only in this working copy          |
| `detached HEAD` | commits made here are easy to lose                           |
| `behind`        | the remote has commits you don't (informational, not a risk) |

Everything except `behind` counts as at risk, and makes `pr0be` exit with code `1` — so it drops straight into a cron job, a git hook, or CI.

### Speed and safety

- Repositories are inspected in parallel; ~24 000 directories take about a second on an M-series Mac.
- Read-only. `pr0be` never commits, pushes, cleans, or writes to your repos — the only network call it can make is the opt-in `--fetch`.
- `--fetch` runs with terminal prompts disabled and a per-repo timeout (default 20s), so a repository that wants credentials is skipped instead of hanging the sweep.
- Directories *inside* a repository are never reported as `not a repo` — that repo already covers them — but repositories nested inside another repository are inspected in their own right.

## Configuration

Config is looked up in `./pr0be.toml`, `~/.config/pr0be/config.toml`, then `~/.pr0be.toml`; the first match wins. `pr0be --init` prints a starting point.

```toml
roots = ["~/Development"]
max_depth = 8

# Directory names never descended into.
ignore = ["node_modules", "vendor", "target", "dist", ".venv"]

# A directory holding one of these but no .git is reported as "not a repo".
# Plain names or a "*.suffix" pattern.
project_markers = ["package.json", "Cargo.toml", "composer.json", "go.mod", "*.xcodeproj"]

# Fetch remotes before checking (slow, needs network). Same as --fetch.
fetch = false
fetch_timeout = 20

# Native notification when problems are found. Same as --notify.
notify = false
```

Command line flags always win over the config file.

## Run it daily

A `launchd` agent at `~/Library/LaunchAgents/com.coderscantina.pr0be.plist` — the notification only fires when something is actually at risk, so a quiet day stays quiet:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.coderscantina.pr0be</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Users/you/.cargo/bin/pr0be</string>
    <string>--notify</string>
  </array>
  <key>StartCalendarInterval</key>
  <dict><key>Hour</key><integer>18</integer><key>Minute</key><integer>0</integer></dict>
  <key>StandardOutPath</key><string>/tmp/pr0be.log</string>
  <key>StandardErrorPath</key><string>/tmp/pr0be.log</string>
</dict>
</plist>
```

```sh
launchctl load ~/Library/LaunchAgents/com.coderscantina.pr0be.plist
```

On Linux, the equivalent crontab line:

```cron
0 18 * * * /home/you/.cargo/bin/pr0be --notify >> /tmp/pr0be.log 2>&1
```

## Scripting

`--json` emits the full result, including clean projects when combined with `--all`:

```console
$ pr0be --json | jq '.entries[] | select(.issues[].kind == "unpushed") | .path'
```

```json
{
  "config": "/Users/you/.config/pr0be/config.toml",
  "scanned_dirs": 24141,
  "projects": 136,
  "at_risk": 106,
  "elapsed_ms": 1047,
  "entries": [
    {
      "path": "/Users/you/Development/tools/phpantom_lsp",
      "is_repo": true,
      "issues": [{ "kind": "dirty", "staged": 0, "modified": 2, "untracked": 0 }]
    }
  ]
}
```

Exit codes: `0` nothing at risk · `1` at least one project holds local-only code · `2` bad configuration.

## Contributing

Bug reports, ideas, and pull requests are welcome — see the [Contributing Guide](CONTRIBUTING.md) for the commit format and PR process, and the [Code of Conduct](CODE_OF_CONDUCT.md) for how we treat each other.

```sh
cargo test
cargo clippy --all-targets
cargo fmt
```

## Security

Please report vulnerabilities responsibly — see the [Security Policy](SECURITY.md).

## License

`pr0be` is licensed under the [MIT License](LICENSE).

## Also from Coder's Cantina

- **[b10cks CMS](https://www.b10cks.com)** — the opinionated headless CMS ([repo](https://github.com/b10cks/cms))
- **[RAVN](https://www.ravnmail.com)** — the email client ([repo](https://github.com/ravnmail/ravn))
- **[gh0st](https://github.com/coderscantina/gh0st)** — the SEO crawler

---

<div align="center">
  <p>Maintained by <a href="https://github.com/badmike">Michael Wallner</a> at <a href="https://www.coderscantina.com">Coder's Cantina</a></p>
</div>
