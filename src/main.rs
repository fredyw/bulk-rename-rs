//! Main entry point for the `bmv` CLI tool.
extern crate bmv;
extern crate clap;

use bmv::{BulkRename, Callback, CollisionStrategy, HistoryCallback, RenameHistory};
use clap::Parser;
use std::collections::HashSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Set the directory.
    #[arg(short = 'f', long)]
    dir: PathBuf,

    /// Set the regex.
    #[arg(short = 'r', long, required_unless_present = "undo")]
    regex: Option<String>,

    /// Set the replacement.
    #[arg(short = 'p', long, required_unless_present = "undo")]
    replacement: Option<String>,

    /// Perform a dry-run.
    #[arg(short = 'd', long, default_value_t = false)]
    dry_run: bool,

    /// Run in quiet mode.
    #[arg(short = 'q', long, default_value_t = false)]
    quiet: bool,

    /// Set the collision strategy.
    #[arg(short = 'c', long, default_value = "skip")]
    collision: CollisionStrategy,

    /// Undo the previous rename operation using the history file.
    #[arg(long, default_value_t = false)]
    undo: bool,

    /// Set the history file path for undo/rollback.
    #[arg(long, default_value = ".bmv-undo.json")]
    history_file: PathBuf,

    /// Prompt for confirmation before each rename.
    #[arg(short = 'i', long, default_value_t = false)]
    interactive: bool,

    /// Use case-insensitive matching.
    #[arg(short = 'I', long, default_value_t = false)]
    ignore_case: bool,

    /// Filter files by extension (comma-separated).
    #[arg(short = 'e', long, value_delimiter = ',')]
    ext: Vec<String>,
}

/// A callback implementation for the CLI.
struct CliCallback {
    /// Whether to suppress output.
    quiet: bool,
}

impl CliCallback {
    /// Creates a new `CliCallback`.
    fn new(quiet: bool) -> Self {
        Self { quiet }
    }
}

impl Callback for CliCallback {
    fn on_ok(&self, old_path: &Path, new_path: &Path) {
        if !self.quiet {
            println!("OK: {} --> {}", old_path.display(), new_path.display());
        }
    }

    fn on_error(&self, old_path: &Path, new_path: &Path, error: std::io::Error) {
        if !self.quiet {
            eprintln!(
                "Error: Unable to rename {} to {}: {}",
                old_path.display(),
                new_path.display(),
                error
            );
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.undo {
        let content = std::fs::read_to_string(&args.history_file)?;
        let history = serde_json::from_str::<RenameHistory>(&content)?;
        BulkRename::undo(&history, CliCallback::new(args.quiet));
        return Ok(());
    }

    let path = args.dir.as_path();
    let regex = args.regex.as_deref().unwrap_or("");
    let replacement = args.replacement.as_deref().unwrap_or("");

    let bulk_rename = BulkRename::new(path, regex, replacement)?;
    let bulk_rename = bulk_rename
        .with_collision_strategy(args.collision)
        .with_case_insensitive(args.ignore_case)?
        .with_extensions(args.ext.into_iter().collect());

    if args.dry_run {
        let targets = Mutex::new(HashSet::new());
        bulk_rename.bulk_rename_fn(|old_path, new_path| {
            if let Some(final_path) = bulk_rename.resolve_collision(old_path, new_path, &targets) {
                println!(
                    "Dry-run: {} --> {}",
                    old_path.display(),
                    final_path.display()
                );
            } else {
                println!("Dry-run: {} (skipped due to collision)", old_path.display());
            }
        });
    } else if args.interactive {
        let targets = Mutex::new(HashSet::new());
        let history = Mutex::new(Vec::new());
        let callback = HistoryCallback::new(CliCallback::new(args.quiet), &history);

        bulk_rename.bulk_rename_fn_seq(|old_path, new_path| {
            if let Some(final_path) = bulk_rename.resolve_collision(old_path, new_path, &targets) {
                if confirm(old_path, &final_path) {
                    match std::fs::rename(old_path, &final_path) {
                        Ok(_) => callback.on_ok(old_path, &final_path),
                        Err(e) => callback.on_error(old_path, &final_path, e),
                    }
                }
            }
        });

        save_history(&args.history_file, history.into_inner().unwrap())?;
    } else {
        let history = Mutex::new(Vec::new());
        let callback = HistoryCallback::new(CliCallback::new(args.quiet), &history);
        bulk_rename.bulk_rename(callback);

        save_history(&args.history_file, history.into_inner().unwrap())?;
    }

    Ok(())
}

fn confirm(old_path: &Path, new_path: &Path) -> bool {
    print!(
        "Rename {} to {}? [y/N] ",
        old_path.display(),
        new_path.display()
    );
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => input.trim().to_lowercase() == "y",
        Err(_) => false,
    }
}

fn save_history(
    history_file: &Path,
    records: Vec<bmv::models::RenameRecord>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !records.is_empty() {
        let history = RenameHistory { records };
        let json = serde_json::to_string_pretty(&history)?;
        std::fs::write(history_file, json)?;
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
