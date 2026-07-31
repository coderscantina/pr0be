use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// Directory trees to scan.
    pub roots: Vec<PathBuf>,
    /// Directory names never descended into.
    pub ignore: Vec<String>,
    /// Files that mark a directory as "a project" even without a .git.
    pub project_markers: Vec<String>,
    /// How deep below each root we look (root itself is depth 0).
    pub max_depth: usize,
    /// Run `git fetch` before inspecting a repo.
    pub fetch: bool,
    /// Send a native notification when problems are found.
    pub notify: bool,
    /// Seconds a single `git fetch` may take before it is given up on.
    pub fetch_timeout: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            roots: vec![home().join("Development")],
            ignore: [
                ".git",
                "node_modules",
                "vendor",
                "target",
                "dist",
                "build",
                ".next",
                ".nuxt",
                ".output",
                ".venv",
                "venv",
                "__pycache__",
                "Pods",
                ".terraform",
                "DerivedData",
                ".cache",
                ".Trash",
                "Library",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            project_markers: [
                "package.json",
                "Cargo.toml",
                "composer.json",
                "go.mod",
                "pyproject.toml",
                "requirements.txt",
                "Gemfile",
                "pom.xml",
                "build.gradle",
                "Makefile",
                "CMakeLists.txt",
                "docker-compose.yml",
                "Package.swift",
                "*.xcodeproj",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            max_depth: 8,
            fetch: false,
            notify: false,
            fetch_timeout: 20,
        }
    }
}

impl Config {
    /// Loads `path` if given, otherwise the first config found in the default
    /// locations. Returns the defaults when nothing exists.
    pub fn load(explicit: Option<&Path>) -> Result<(Self, Option<PathBuf>), String> {
        let path = match explicit {
            Some(p) => Some(p.to_path_buf()),
            None => default_paths().into_iter().find(|p| p.is_file()),
        };
        let Some(path) = path else {
            return Ok((Self::default(), None));
        };
        let raw = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let cfg: Self = toml::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok((cfg, Some(path)))
    }

    pub fn expanded_roots(&self) -> Vec<PathBuf> {
        self.roots.iter().map(|r| expand_tilde(r)).collect()
    }
}

fn default_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("pr0be.toml"),
        home().join(".config/pr0be/config.toml"),
        home().join(".pr0be.toml"),
    ]
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}

pub fn expand_tilde(p: &Path) -> PathBuf {
    match p.strip_prefix("~") {
        Ok(rest) => home().join(rest),
        Err(_) => p.to_path_buf(),
    }
}

pub const EXAMPLE: &str = r#"# ~/.config/pr0be/config.toml
roots = ["~/Development"]
max_depth = 8

# Never descended into.
ignore = ["node_modules", "vendor", "target", "dist", ".venv"]

# A directory holding one of these but no .git is reported as "not a repo".
project_markers = ["package.json", "Cargo.toml", "composer.json", "go.mod"]

# Fetch remotes before checking (slow, needs network). Same as --fetch.
fetch = false
fetch_timeout = 20

# Native notification when problems are found. Same as --notify.
notify = false
"#;
