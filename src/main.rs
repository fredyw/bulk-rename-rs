//! Main entry point for the `bmv` CLI tool.
extern crate bmv;
extern crate clap;

use bmv::bulk_rename::BulkRename;
use bmv::bulk_rename::Callback;
use bmv::bulk_rename::CollisionStrategy;
use bmv::bulk_rename::Error;
use clap::Parser;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Set the directory.
    #[arg(short = 'f', long)]
    dir: PathBuf,

    /// Set the regex.
    #[arg(short = 'r', long)]
    regex: String,

    /// Set the replacement.
    #[arg(short = 'p', long)]
    replacement: String,

    /// Perform a dry-run.
    #[arg(short = 'd', long, default_value_t = false)]
    dry_run: bool,

    /// Run in quiet mode.
    #[arg(short = 'q', long, default_value_t = false)]
    quiet: bool,

    /// Set the collision strategy.
    #[arg(short = 'c', long, default_value = "skip", value_parser = ["skip", "overwrite", "suffix"])]
    collision: CollisionStrategy,
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

/// Main entry point for the application.
fn main() {
    let args = Args::parse();
    let path = args.dir.as_path();
    let bulk_rename = BulkRename::new(path, &args.regex, &args.replacement);
    match bulk_rename {
        Ok(bulk_rename) => {
            let bulk_rename = bulk_rename.with_collision_strategy(args.collision);
            if args.dry_run {
                let targets = Mutex::new(HashSet::new());
                bulk_rename.bulk_rename_fn(|old_path, new_path| {
                    if let Some(final_path) =
                        bulk_rename.resolve_collision(old_path, new_path, &targets)
                    {
                        println!(
                            "Dry-run: {} --> {}",
                            old_path.display(),
                            final_path.display()
                        );
                    } else {
                        println!("Dry-run: {} (skipped due to collision)", old_path.display());
                    }
                })
            } else {
                bulk_rename.bulk_rename(CliCallback::new(args.quiet))
            }
        }
        Err(error) => match error {
            Error::NotDirError => {
                eprintln!("Error: {} is not a directory", path.display())
            }
            Error::RegexError(error) => {
                eprintln!("Error: {} is not a valid regex: '{}'", args.regex, error)
            }
        },
    }
}
