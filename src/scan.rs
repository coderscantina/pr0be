use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::Config;

#[derive(Debug)]
pub struct Candidates {
    pub repos: Vec<PathBuf>,
    /// Project-looking directories without any git repository.
    pub loose: Vec<PathBuf>,
    pub dirs_seen: usize,
}

/// Walks every configured root and collects git repos plus project directories
/// that are not under git control. Nested repos are found too: a repo does not
/// stop the walk, only the ignore list and `max_depth` do.
pub fn scan(cfg: &Config) -> Candidates {
    let ignore: HashSet<&str> = cfg.ignore.iter().map(String::as_str).collect();
    let mut out = Candidates {
        repos: Vec::new(),
        loose: Vec::new(),
        dirs_seen: 0,
    };

    for root in cfg.expanded_roots() {
        if !root.is_dir() {
            eprintln!("warning: root {} does not exist", root.display());
            continue;
        }
        walk(&root, 0, false, cfg, &ignore, &mut out);
    }

    out.repos.sort();
    out.loose.sort();
    out
}

fn walk(
    dir: &Path,
    depth: usize,
    in_repo: bool,
    cfg: &Config,
    ignore: &HashSet<&str>,
    out: &mut Candidates,
) {
    out.dirs_seen += 1;

    let is_repo = dir.join(".git").exists();
    if is_repo {
        out.repos.push(dir.to_path_buf());
    } else if !in_repo && has_marker(dir, cfg) {
        // A project without git — record it and stop; its subdirectories are
        // part of that project, not separate ones. Inside a repo we keep
        // walking instead: such directories are already covered by that repo,
        // but nested repositories below them are not.
        out.loose.push(dir.to_path_buf());
        return;
    }

    if depth >= cfg.max_depth {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        if !ft.is_dir() || ft.is_symlink() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if ignore.contains(name.as_ref()) {
            continue;
        }
        walk(
            &entry.path(),
            depth + 1,
            in_repo || is_repo,
            cfg,
            ignore,
            out,
        );
    }
}

/// True when the directory contains one of the configured project markers.
/// Markers may be plain names or a `*.ext` suffix pattern.
fn has_marker(dir: &Path, cfg: &Config) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    let names: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    cfg.project_markers
        .iter()
        .any(|marker| match marker.strip_prefix('*') {
            Some(suffix) => names.iter().any(|n| n.ends_with(suffix)),
            None => names.iter().any(|n| n == marker),
        })
}
