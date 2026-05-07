use rayon::prelude::*;
use regex::Regex;
use std::borrow::Cow;
use std::path::Path;
use std::{fs, io};
use walkdir::WalkDir;

/// A bulk rename operation.
pub struct BulkRename<'a> {
    /// The directory to search for files.
    dir: &'a Path,
    /// The regular expression to match against file names.
    regex: Regex,
    /// The replacement string for matched file names.
    replacement: &'a str,
}

/// Possible errors when running a bulk rename.
#[derive(Debug)]
pub enum Error {
    /// The provided path is not a directory.
    NotDirError,
    /// The provided regular expression is invalid.
    RegexError(regex::Error),
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
        })
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
        self.bulk_rename_fn(|old_path, new_path| match fs::rename(old_path, new_path) {
            Ok(_) => {
                callback.on_ok(old_path, new_path);
            }
            Err(error) => {
                callback.on_error(old_path, new_path, error);
            }
        })
    }
}
