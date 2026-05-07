use rayon::prelude::*;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;
use std::{fs, io};
use walkdir::WalkDir;

/// A bulk rename operation.
#[derive(Debug)]
pub struct BulkRename<'a> {
    /// The directory to search for files.
    dir: &'a Path,
    /// The regular expression to match against file names.
    regex: Regex,
    /// The replacement string for matched file names.
    replacement: &'a str,
    /// The strategy for handling collisions.
    collision_strategy: CollisionStrategy,
}

/// Possible errors when running a bulk rename.
#[derive(Debug)]
pub enum Error {
    /// The provided path is not a directory.
    NotDirError,
    /// The provided regular expression is invalid.
    RegexError(regex::Error),
}

/// Strategies for handling filename collisions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CollisionStrategy {
    /// Skip the rename if the destination already exists.
    #[default]
    Skip,
    /// Overwrite the destination if it already exists.
    Overwrite,
    /// Append a suffix to the filename if the destination already exists.
    Suffix,
}

impl FromStr for CollisionStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "skip" => Ok(CollisionStrategy::Skip),
            "overwrite" => Ok(CollisionStrategy::Overwrite),
            "suffix" => Ok(CollisionStrategy::Suffix),
            _ => Err(format!(
                "invalid collision strategy: {}. Valid values are: skip, overwrite, suffix",
                s
            )),
        }
    }
}

impl fmt::Display for CollisionStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CollisionStrategy::Skip => "skip",
            CollisionStrategy::Overwrite => "overwrite",
            CollisionStrategy::Suffix => "suffix",
        };
        write!(f, "{}", s)
    }
}

/// A callback for the bulk rename.
pub trait Callback: Sync + Send {
    /// Called when a file rename operation was successful.
    fn on_ok(&self, old_path: &Path, new_path: &Path);

    /// Called when a file rename operation failed.
    fn on_error(&self, old_path: &Path, new_path: &Path, error: io::Error);
}

/// A no-op `Callback`.
pub struct NoOpCallback {}

impl Default for NoOpCallback {
    fn default() -> Self {
        Self::new()
    }
}

impl NoOpCallback {
    /// Creates a new no-op `Callback`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Callback for NoOpCallback {
    fn on_ok(&self, _old_path: &Path, _new_path: &Path) {}

    fn on_error(&self, _old_path: &Path, _new_path: &Path, _error: io::Error) {}
}

impl<'a> BulkRename<'a> {
    /// Creates a new `BulkRename`.
    pub fn new(dir: &'a Path, regex: &'a str, replacement: &'a str) -> Result<Self, Error> {
        if !dir.is_dir() {
            return Err(Error::NotDirError);
        }
        let regex = Regex::new(regex).map_err(Error::RegexError)?;
        Ok(Self {
            dir,
            regex,
            replacement,
            collision_strategy: CollisionStrategy::default(),
        })
    }

    /// Sets the collision strategy.
    pub fn with_collision_strategy(mut self, strategy: CollisionStrategy) -> Self {
        self.collision_strategy = strategy;
        self
    }

    /// Executes a function `f` for any files that match the specified regex.
    ///
    /// The function `f` is called with the original path and the calculated new path.
    /// It will not be called if the file name remains unchanged after replacement.
    /// This operation is performed in parallel across multiple threads.
    pub fn bulk_rename_fn<F>(&self, f: F)
    where
        F: Fn(&Path, &Path) + Sync + Send,
    {
        WalkDir::new(self.dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .par_bridge()
            .for_each(|entry| {
                let path = entry.path();
                if let Some(old_file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let new_file_name = self.regex.replace_all(old_file_name, self.replacement);
                    if let Cow::Owned(new_name) = new_file_name {
                        if old_file_name != new_name {
                            let mut new_path = path.to_path_buf();
                            new_path.set_file_name(new_name);
                            f(path, &new_path);
                        }
                    }
                }
            });
    }

    /// Performs the bulk rename operation, notifying the provided `callback` of each outcome.
    ///
    /// Files are renamed in place. This operation is performed in parallel across multiple threads.
    pub fn bulk_rename(&self, callback: impl Callback) {
        let targets = Mutex::new(HashSet::new());
        self.bulk_rename_fn(|old_path, new_path| {
            let final_path = match self.resolve_collision(old_path, new_path, &targets) {
                Some(path) => path,
                None => return, // Skip
            };

            match fs::rename(old_path, &final_path) {
                Ok(_) => {
                    callback.on_ok(old_path, &final_path);
                }
                Err(error) => {
                    callback.on_error(old_path, &final_path, error);
                }
            }
        })
    }

    /// Resolves a collision for a given path and target path.
    ///
    /// Returns `Some(PathBuf)` if the rename should proceed, or `None` if it should be skipped.
    pub fn resolve_collision(
        &self,
        old_path: &Path,
        new_path: &Path,
        targets: &Mutex<HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        let mut final_path = new_path.to_path_buf();

        match self.collision_strategy {
            CollisionStrategy::Skip => {
                if final_path.exists() && !Self::is_same_file(old_path, &final_path) {
                    return None;
                }
                let mut t = targets.lock().unwrap();
                if t.contains(&final_path) {
                    return None;
                }
                t.insert(final_path.clone());
            }
            CollisionStrategy::Overwrite => {
                // Do nothing, just return new_path.
                // Note: We don't track targets here because overwrite allows collisions.
            }
            CollisionStrategy::Suffix => {
                let mut i = 1;
                let stem = new_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let ext = new_path.extension().and_then(|e| e.to_str());

                loop {
                    let mut t = targets.lock().unwrap();
                    let exists = final_path.exists() && !Self::is_same_file(old_path, &final_path);
                    if !exists && !t.contains(&final_path) {
                        t.insert(final_path.clone());
                        break;
                    }
                    drop(t);

                    let new_name = match ext {
                        Some(ext) => format!("{} ({}).{}", stem, i, ext),
                        None => format!("{} ({})", stem, i),
                    };
                    final_path.set_file_name(new_name);
                    i += 1;
                }
            }
        }
        Some(final_path)
    }

    fn is_same_file(p1: &Path, p2: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let (Ok(m1), Ok(m2)) = (fs::metadata(p1), fs::metadata(p2)) {
                return m1.dev() == m2.dev() && m1.ino() == m2.ino();
            }
        }
        #[cfg(windows)]
        {
            // Windows has a different way, but for now we can just check if they are the same path
            // since Windows is case-insensitive usually.
            // But this is just a fallback.
            if let (Ok(p1_canonical), Ok(p2_canonical)) = (p1.canonicalize(), p2.canonicalize()) {
                return p1_canonical == p2_canonical;
            }
        }
        p1 == p2
    }
}
