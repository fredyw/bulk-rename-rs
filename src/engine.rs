use crate::callback::Callback;
use crate::error::Error;
use crate::models::{CollisionStrategy, RenameHistory};
use rayon::prelude::*;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
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
    /// The extensions to filter by.
    extensions: HashSet<String>,
    /// The patterns to include.
    include_patterns: Vec<Regex>,
    /// The patterns to exclude.
    exclude_patterns: Vec<Regex>,
}

impl<'a> BulkRename<'a> {
    /// Creates a new `BulkRename`.
    pub fn new(dir: &'a Path, regex: &'a str, replacement: &'a str) -> Result<Self, Error> {
        if !dir.is_dir() {
            return Err(Error::NotDirError);
        }
        let regex = Regex::new(regex)?;
        Ok(Self {
            dir,
            regex,
            replacement,
            collision_strategy: CollisionStrategy::default(),
            extensions: HashSet::new(),
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
        })
    }

    /// Sets the collision strategy.
    pub fn with_collision_strategy(mut self, strategy: CollisionStrategy) -> Self {
        self.collision_strategy = strategy;
        self
    }

    /// Sets whether the regex matching should be case-insensitive.
    pub fn with_case_insensitive(mut self, ignore_case: bool) -> Result<Self, Error> {
        if ignore_case {
            let pattern = self.regex.as_str();
            self.regex = regex::RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()?;
        }
        Ok(self)
    }

    /// Sets the extensions to filter by.
    pub fn with_extensions(mut self, extensions: HashSet<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Sets the patterns to include.
    pub fn with_include_patterns(mut self, patterns: Vec<String>) -> Result<Self, Error> {
        self.include_patterns = patterns
            .into_iter()
            .map(|p| Regex::new(&p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self)
    }

    /// Sets the patterns to exclude.
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Result<Self, Error> {
        self.exclude_patterns = patterns
            .into_iter()
            .map(|p| Regex::new(&p))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self)
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
            .filter(|entry| {
                if self.extensions.is_empty() {
                    return true;
                }
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| self.extensions.contains(ext))
                    .unwrap_or(false)
            })
            .filter(|entry| {
                let path_str = entry.path().to_string_lossy();
                if !self.exclude_patterns.is_empty()
                    && self
                        .exclude_patterns
                        .iter()
                        .any(|re| re.is_match(&path_str))
                {
                    return false;
                }
                if !self.include_patterns.is_empty()
                    && !self
                        .include_patterns
                        .iter()
                        .any(|re| re.is_match(&path_str))
                {
                    return false;
                }
                true
            })
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

    /// Sequential version of `bulk_rename_fn`.
    pub fn bulk_rename_fn_seq<F>(&self, mut f: F)
    where
        F: FnMut(&Path, &Path),
    {
        WalkDir::new(self.dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                if self.extensions.is_empty() {
                    return true;
                }
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| self.extensions.contains(ext))
                    .unwrap_or(false)
            })
            .filter(|entry| {
                let path_str = entry.path().to_string_lossy();
                if !self.exclude_patterns.is_empty()
                    && self
                        .exclude_patterns
                        .iter()
                        .any(|re| re.is_match(&path_str))
                {
                    return false;
                }
                if !self.include_patterns.is_empty()
                    && !self
                        .include_patterns
                        .iter()
                        .any(|re| re.is_match(&path_str))
                {
                    return false;
                }
                true
            })
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
            if let (Ok(p1_canonical), Ok(p2_canonical)) = (p1.canonicalize(), p2.canonicalize()) {
                return p1_canonical == p2_canonical;
            }
        }
        p1 == p2
    }

    /// Undoes the renames specified in the given `history`.
    pub fn undo(history: &RenameHistory, callback: impl Callback) {
        history.records.par_iter().for_each(|record| {
            match fs::rename(&record.new_path, &record.old_path) {
                Ok(_) => {
                    callback.on_ok(&record.new_path, &record.old_path);
                }
                Err(error) => {
                    callback.on_error(&record.new_path, &record.old_path, error);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_bulk_rename_new() {
        let dir = tempdir().unwrap();
        let bulk_rename = BulkRename::new(dir.path(), ".*", "replacement").unwrap();
        assert_eq!(bulk_rename.dir, dir.path());
        assert_eq!(bulk_rename.replacement, "replacement");
        assert_eq!(bulk_rename.collision_strategy, CollisionStrategy::Skip);
    }

    #[test]
    fn test_bulk_rename_new_not_dir() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file");
        File::create(&file_path).unwrap();
        let result = BulkRename::new(&file_path, ".*", "replacement");
        assert!(matches!(result, Err(Error::NotDirError)));
    }

    #[test]
    fn test_bulk_rename_new_invalid_regex() {
        let dir = tempdir().unwrap();
        let result = BulkRename::new(dir.path(), "[", "replacement");
        assert!(result.is_err());
    }

    #[test]
    fn test_bulk_rename_with_collision_strategy() {
        let dir = tempdir().unwrap();
        let bulk_rename = BulkRename::new(dir.path(), ".*", "replacement")
            .unwrap()
            .with_collision_strategy(CollisionStrategy::Overwrite);
        assert_eq!(bulk_rename.collision_strategy, CollisionStrategy::Overwrite);
    }

    #[test]
    fn test_is_same_file() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1");
        let file2 = dir.path().join("file2");
        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        assert!(BulkRename::is_same_file(&file1, &file1));
        assert!(!BulkRename::is_same_file(&file1, &file2));
    }

    #[test]
    fn test_resolve_collision_skip() {
        let dir = tempdir().unwrap();
        let old_path = dir.path().join("old");
        let new_path = dir.path().join("new");
        File::create(&new_path).unwrap();

        let bulk_rename = BulkRename::new(dir.path(), ".*", "replacement").unwrap();
        let targets = Mutex::new(HashSet::new());

        let result = bulk_rename.resolve_collision(&old_path, &new_path, &targets);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_collision_overwrite() {
        let dir = tempdir().unwrap();
        let old_path = dir.path().join("old");
        let new_path = dir.path().join("new");
        File::create(&new_path).unwrap();

        let bulk_rename = BulkRename::new(dir.path(), ".*", "replacement")
            .unwrap()
            .with_collision_strategy(CollisionStrategy::Overwrite);
        let targets = Mutex::new(HashSet::new());

        let result = bulk_rename.resolve_collision(&old_path, &new_path, &targets);
        assert_eq!(result.unwrap(), new_path);
    }

    #[test]
    fn test_resolve_collision_suffix() {
        let dir = tempdir().unwrap();
        let old_path = dir.path().join("old");
        let new_path = dir.path().join("new.txt");
        File::create(&new_path).unwrap();

        let bulk_rename = BulkRename::new(dir.path(), ".*", "replacement")
            .unwrap()
            .with_collision_strategy(CollisionStrategy::Suffix);
        let targets = Mutex::new(HashSet::new());

        let result = bulk_rename.resolve_collision(&old_path, &new_path, &targets);
        assert_eq!(result.unwrap(), dir.path().join("new (1).txt"));
    }
}
