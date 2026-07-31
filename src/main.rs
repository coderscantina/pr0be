mod config;
mod git;
mod notify;
mod report;
mod scan;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::Parser;
use rayon::prelude::*;

use config::Config;
use git::{Entry, Issue};
use report::Style;

/// Find code that only exists on this machine.
#[derive(Parser, Debug)]
#[command(name = "pr0be", version, about, long_about = None)]
struct Cli {
    /// Directory trees to scan (overrides the config roots).
    roots: Vec<PathBuf>,

    /// Config file to use instead of the default locations.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Fetch remotes before checking, so "behind" is accurate.
    #[arg(long)]
    fetch: bool,

    /// Show clean projects too.
    #[arg(short, long)]
    all: bool,

    /// Machine readable output.
    #[arg(long)]
    json: bool,

    /// Send a desktop notification when problems are found.
    #[arg(short, long)]
    notify: bool,

    /// How deep to descend below each root.
    #[arg(long, value_name = "N")]
    max_depth: Option<usize>,

    /// Never colorize output.
    #[arg(long)]
    no_color: bool,

    /// Print an example config file and exit.
    #[arg(long)]
    init: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.init {
        print!("{}", config::EXAMPLE);
        return ExitCode::SUCCESS;
    }

    let (mut cfg, cfg_path) = match Config::load(cli.config.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };

    if !cli.roots.is_empty() {
        cfg.roots = cli.roots.clone();
    }
    if let Some(d) = cli.max_depth {
        cfg.max_depth = d;
    }
    cfg.fetch |= cli.fetch;
    cfg.notify |= cli.notify;

    let started = Instant::now();
    let candidates = scan::scan(&cfg);
    let timeout = Duration::from_secs(cfg.fetch_timeout);

    let mut entries: Vec<Entry> = candidates
        .repos
        .par_iter()
        .map(|p| git::inspect(p, cfg.fetch, timeout))
        .collect();

    entries.extend(candidates.loose.iter().map(|p| Entry {
        path: p.clone(),
        is_repo: false,
        issues: vec![Issue::NotARepo],
    }));
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let flagged: Vec<Entry> = entries
        .iter()
        .filter(|e| cli.all || !e.issues.is_empty())
        .cloned()
        .collect();
    let at_risk = entries.iter().filter(|e| e.has_risk()).count();
    let elapsed_ms = started.elapsed().as_millis();

    if cli.json {
        let doc = serde_json::json!({
            "config": cfg_path.map(|p| p.display().to_string()),
            "scanned_dirs": candidates.dirs_seen,
            "projects": entries.len(),
            "at_risk": at_risk,
            "elapsed_ms": elapsed_ms,
            "entries": flagged,
        });
        println!("{}", serde_json::to_string_pretty(&doc).unwrap());
    } else {
        let style = Style {
            color: !cli.no_color
                && std::env::var_os("NO_COLOR").is_none()
                && std::io::IsTerminal::is_terminal(&std::io::stdout()),
        };
        report::print_table(
            &flagged,
            entries.len(),
            &style,
            candidates.dirs_seen,
            elapsed_ms,
        );
    }

    if cfg.notify && at_risk > 0 {
        let (title, body) = report::notification_text(&flagged, entries.len());
        notify::send(&title, &body);
    }

    if at_risk > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
