use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Issue {
    /// Looks like a project but has no git repository at all.
    NotARepo,
    /// A repo without any remote — nothing to push to.
    NoRemote,
    /// Working tree has changes that were never committed.
    Dirty {
        staged: usize,
        modified: usize,
        untracked: usize,
    },
    /// Branch exists only here, no upstream configured.
    NoUpstream { branch: String },
    /// Branch has commits the remote does not have.
    Unpushed { branch: String, ahead: usize },
    /// Upstream is configured but gone from the remote.
    UpstreamGone { branch: String },
    /// Stashed work — lives only in this working copy.
    Stashed { count: usize },
    /// Commits on a detached HEAD are easy to lose.
    DetachedHead { head: String },
    /// Remote has commits we don't (informational — not a backup risk).
    Behind { branch: String, behind: usize },
    /// git itself failed here.
    Failed { detail: String },
}

impl Issue {
    /// Whether this means code exists *only* on this machine.
    pub fn is_risk(&self) -> bool {
        !matches!(self, Issue::Behind { .. })
    }

    pub fn label(&self) -> &'static str {
        match self {
            Issue::NotARepo => "not a repo",
            Issue::NoRemote => "no remote",
            Issue::Dirty { .. } => "uncommitted",
            Issue::NoUpstream { .. } => "no upstream",
            Issue::Unpushed { .. } => "unpushed",
            Issue::UpstreamGone { .. } => "upstream gone",
            Issue::Stashed { .. } => "stashed",
            Issue::DetachedHead { .. } => "detached HEAD",
            Issue::Behind { .. } => "behind",
            Issue::Failed { .. } => "error",
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Issue::NotARepo => "project directory is not under git control".into(),
            Issue::NoRemote => "no remote configured — nothing to push to".into(),
            Issue::Dirty {
                staged,
                modified,
                untracked,
            } => {
                let mut parts = Vec::new();
                if *staged > 0 {
                    parts.push(format!("{staged} staged"));
                }
                if *modified > 0 {
                    parts.push(format!("{modified} modified"));
                }
                if *untracked > 0 {
                    parts.push(format!("{untracked} untracked"));
                }
                parts.join(", ")
            }
            Issue::NoUpstream { branch } => format!("{branch} has no upstream"),
            Issue::Unpushed { branch, ahead } => {
                format!("{branch} is {ahead} commit(s) ahead")
            }
            Issue::UpstreamGone { branch } => format!("{branch}: upstream is gone"),
            Issue::Stashed { count } => format!("{count} stash entry/entries"),
            Issue::DetachedHead { head } => format!("detached at {head}"),
            Issue::Behind { branch, behind } => {
                format!("{branch} is {behind} commit(s) behind")
            }
            Issue::Failed { detail } => detail.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub path: PathBuf,
    pub is_repo: bool,
    pub issues: Vec<Issue>,
}

impl Entry {
    pub fn has_risk(&self) -> bool {
        self.issues.iter().any(Issue::is_risk)
    }
}

/// Runs git in `dir` and returns stdout, or an error string.
fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!("git {}: {err}", args.join(" ")));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `git fetch` with a hard timeout so an auth prompt or dead host can't hang a scan.
fn fetch(dir: &Path, timeout: Duration) -> Result<(), String> {
    let mut child = Command::new("git")
        .current_dir(dir)
        .args(["fetch", "--all", "--prune", "--quiet"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "true")
        .env("SSH_ASKPASS", "true")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("git fetch: {e}"))?;

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err("git fetch failed".into())
                };
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("git fetch timed out after {}s", timeout.as_secs()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(format!("git fetch: {e}")),
        }
    }
}

/// Inspects one repository.
pub fn inspect(path: &Path, do_fetch: bool, fetch_timeout: Duration) -> Entry {
    let mut issues = Vec::new();

    if do_fetch {
        if let Err(e) = fetch(path, fetch_timeout) {
            issues.push(Issue::Failed { detail: e });
        }
    }

    match git(path, &["status", "--porcelain", "-uall"]) {
        Ok(out) => {
            let (mut staged, mut modified, mut untracked) = (0, 0, 0);
            for line in out.lines() {
                let bytes = line.as_bytes();
                if bytes.len() < 2 {
                    continue;
                }
                match (bytes[0], bytes[1]) {
                    (b'?', _) => untracked += 1,
                    (x, y) => {
                        if x != b' ' {
                            staged += 1;
                        }
                        if y != b' ' {
                            modified += 1;
                        }
                    }
                }
            }
            if staged + modified + untracked > 0 {
                issues.push(Issue::Dirty {
                    staged,
                    modified,
                    untracked,
                });
            }
        }
        Err(e) => issues.push(Issue::Failed { detail: e }),
    }

    let has_remote = git(path, &["remote"])
        .map(|o| !o.trim().is_empty())
        .unwrap_or(false);
    if !has_remote {
        issues.push(Issue::NoRemote);
    }

    if let Ok(out) = git(path, &["stash", "list"]) {
        let count = out.lines().filter(|l| !l.trim().is_empty()).count();
        if count > 0 {
            issues.push(Issue::Stashed { count });
        }
    }

    if git(path, &["symbolic-ref", "-q", "HEAD"]).is_err() {
        let head = git(path, &["rev-parse", "--short", "HEAD"])
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if !head.is_empty() {
            issues.push(Issue::DetachedHead { head });
        }
    }

    let refs = git(
        path,
        &[
            "for-each-ref",
            "--format=%(refname:short)\t%(upstream:short)\t%(upstream:track)",
            "refs/heads",
        ],
    );
    if let Ok(out) = refs {
        for line in out.lines() {
            let mut cols = line.split('\t');
            let branch = cols.next().unwrap_or("").to_string();
            let upstream = cols.next().unwrap_or("");
            let track = cols.next().unwrap_or("");
            if branch.is_empty() {
                continue;
            }
            if upstream.is_empty() {
                // A branch with no upstream is only a risk if it holds commits
                // no other branch has, but treat it as one when there is no
                // remote at all or it is genuinely unmerged.
                if has_remote {
                    issues.push(Issue::NoUpstream { branch });
                }
                continue;
            }
            if track.contains("gone") {
                issues.push(Issue::UpstreamGone { branch });
                continue;
            }
            if let Some(n) = parse_track(track, "ahead") {
                issues.push(Issue::Unpushed {
                    branch: branch.clone(),
                    ahead: n,
                });
            }
            if let Some(n) = parse_track(track, "behind") {
                issues.push(Issue::Behind { branch, behind: n });
            }
        }
    }

    Entry {
        path: path.to_path_buf(),
        is_repo: true,
        issues,
    }
}

/// Pulls the number out of git's `[ahead 2, behind 1]` tracking string.
fn parse_track(track: &str, word: &str) -> Option<usize> {
    let start = track.find(word)? + word.len();
    track[start..]
        .trim_start()
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::parse_track;

    #[test]
    fn reads_ahead_and_behind() {
        assert_eq!(parse_track("[ahead 2, behind 1]", "ahead"), Some(2));
        assert_eq!(parse_track("[ahead 2, behind 1]", "behind"), Some(1));
        assert_eq!(parse_track("[ahead 3]", "behind"), None);
        assert_eq!(parse_track("", "ahead"), None);
    }
}
