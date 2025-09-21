use rayon::prelude::*;
use regex::Regex;
use std::path::Path;
use std::{fs, io};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct BulkRename<'a> {
    dir: &'a Path,
    regex: Regex,
    replacement: &'a str,
}

/// Possible errors when running a bulk rename.
#[derive(Debug)]
pub enum Error {
    /// An error for when the path is not a directory.
    NotDirError,
    /// An error for when the regex is invalid.
    RegexError(regex::Error),
}

/// A callback for the bulk rename.
pub trait Callback: Sync + Send {
    /// This function is called when the rename operation was successful.
    fn on_ok(&self, old_path: &Path, new_path: &Path);

    /// This function is called when the rename operation was unsuccessful.
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

    /// Executes a function `f` for any files that match the specified regex. The function `f` will
    /// not be called if the old file name is the same as the new file name.
    pub fn bulk_rename_fn<F>(&self, f: F)
    where
        F: Fn(&Path, &Path) + Sync + Send,
    {
        WalkDir::new(self.dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .par_bridge()
            .for_each(|entry| {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if let Some(old_file_name) = file_name.to_str() {
                            let new_file_name = self
                                .regex
                                .replace_all(old_file_name, self.replacement)
                                .to_string();
                            if old_file_name != new_file_name {
                                let mut new_path = path.to_path_buf();
                                new_path.set_file_name(new_file_name);
                                f(path, &new_path);
                            }
                        }
                    }
                }
            });
    }

    /// Runs a bulk rename with a `Callback`.
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
