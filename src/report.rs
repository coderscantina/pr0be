use std::path::Path;

use crate::config::home;
use crate::git::{Entry, Issue};

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";

pub struct Style {
    pub color: bool,
}

impl Style {
    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("{code}{text}{RESET}")
        } else {
            text.to_string()
        }
    }
}

fn color_for(issue: &Issue) -> &'static str {
    match issue {
        Issue::NotARepo | Issue::NoRemote | Issue::UpstreamGone { .. } => RED,
        Issue::Dirty { .. } | Issue::Unpushed { .. } | Issue::NoUpstream { .. } => YELLOW,
        Issue::Stashed { .. } | Issue::DetachedHead { .. } => YELLOW,
        Issue::Behind { .. } => BLUE,
        Issue::Failed { .. } => RED,
    }
}

/// Shortens `/Users/me/Development/x` to `~/Development/x`.
pub fn pretty_path(path: &Path) -> String {
    match path.strip_prefix(home()) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

pub fn print_table(
    entries: &[Entry],
    total: usize,
    style: &Style,
    scanned: usize,
    elapsed_ms: u128,
) {
    if entries.is_empty() {
        println!(
            "{} {scanned} director{} scanned in {elapsed_ms}ms — everything is committed and pushed.",
            style.paint(GREEN, "✓"),
            if scanned == 1 { "y" } else { "ies" }
        );
        return;
    }

    for entry in entries {
        let path = pretty_path(&entry.path);
        let marker = if entry.has_risk() {
            style.paint(RED, "●")
        } else {
            style.paint(BLUE, "○")
        };
        println!("{marker} {}", style.paint(BOLD, &path));
        for issue in &entry.issues {
            println!(
                "  {:<14} {}",
                style.paint(color_for(issue), issue.label()),
                style.paint(DIM, &issue.detail())
            );
        }
        println!();
    }

    print!("{}", summary_line(entries, total, style));
    println!(
        "{}",
        style.paint(
            DIM,
            &format!("  ({scanned} directories scanned in {elapsed_ms}ms)")
        )
    );
}

pub fn summary_line(entries: &[Entry], total: usize, style: &Style) -> String {
    let at_risk = entries.iter().filter(|e| e.has_risk()).count();
    let mut counts = [0usize; 6];
    for issue in entries.iter().flat_map(|e| &e.issues) {
        match issue {
            Issue::NotARepo => counts[0] += 1,
            Issue::Dirty { .. } => counts[1] += 1,
            Issue::Unpushed { .. } | Issue::NoUpstream { .. } => counts[2] += 1,
            Issue::NoRemote => counts[3] += 1,
            Issue::Stashed { .. } => counts[4] += 1,
            Issue::Behind { .. } => counts[5] += 1,
            _ => {}
        }
    }
    let labels = [
        "not a repo",
        "uncommitted",
        "unpushed",
        "no remote",
        "stashed",
        "behind",
    ];
    let details: Vec<String> = counts
        .iter()
        .zip(labels)
        .filter(|(n, _)| **n > 0)
        .map(|(n, l)| format!("{n} {l}"))
        .collect();

    format!(
        "{} {} of {} projects hold code that only exists here: {}\n",
        style.paint(RED, "▲"),
        at_risk,
        total,
        details.join(", ")
    )
}

/// Plain-text version used for the desktop notification.
pub fn notification_text(entries: &[Entry], total: usize) -> (String, String) {
    let plain = Style { color: false };
    let at_risk = entries.iter().filter(|e| e.has_risk()).count();
    let body = summary_line(entries, total, &plain)
        .trim_start_matches("▲ ")
        .trim()
        .to_string();
    (format!("{at_risk} project(s) not backed up"), body)
}
