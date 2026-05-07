use crate::callback::Callback;
use crate::error::Error;
use crate::models::{CollisionStrategy, RenameHistory};
use chrono::{DateTime, Local};
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
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
    /// The maximum depth for recursion.
    max_depth: Option<usize>,
    /// The current counter value for {i}.
    counter: AtomicUsize,
    /// Whether to rename files.
    rename_files: bool,
    /// Whether to rename directories.
    rename_dirs: bool,
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
            max_depth: None,
            counter: AtomicUsize::new(1),
            rename_files: true,
            rename_dirs: false,
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

    /// Sets the maximum depth for recursion.
    pub fn with_max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets the starting value for the counter {i}.
    pub fn with_counter_start(self, start: usize) -> Self {
        self.counter.store(start, Ordering::SeqCst);
        self
    }

    /// Sets whether to rename directories.
    pub fn with_rename_dirs(mut self, rename_dirs: bool) -> Self {
        self.rename_dirs = rename_dirs;
        self
    }

    /// Sets whether to rename files.
    pub fn with_rename_files(mut self, rename_files: bool) -> Self {
        self.rename_files = rename_files;
        self
    }

    /// Executes a function `f` for any files that match the specified regex.
    ///
    /// The function `f` is called with the original path and the calculated new path.
    /// It will not be called if the file name remains unchanged after replacement.
    /// This operation is performed in parallel across multiple threads.
    pub fn run<F>(&self, f: F)
    where
        F: Fn(&Path, &Path) + Sync + Send,
    {
        let plan = self.generate_plan();
        plan.into_par_iter().for_each(|(old, new)| {
            f(&old, &new);
        });
    }

    fn generate_plan(&self) -> Vec<(PathBuf, PathBuf)> {
        let mut walker = WalkDir::new(self.dir);
        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }
        if self.rename_dirs {
            walker = walker.contents_first(true);
        }

        let mut entries: Vec<_> = walker
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                if entry.file_type().is_file() {
                    self.rename_files
                } else if self.rename_dirs && entry.file_type().is_dir() {
                    // Don't rename the root directory
                    entry.depth() > 0
                } else {
                    false
                }
            })
            .filter(|entry| self.filter_entry(entry))
            .collect();

        // Always sort for predictability.
        // If rename_dirs is true, we rely on contents_first(true) which gives us bottom-up.
        // If we sort by path, we might break the bottom-up order if we are not careful.
        // Actually, WalkDir with contents_first(true) already provides a good order.
        // But if we want to be predictable across runs, we might want to sort, but we must maintain depth.

        if !self.rename_dirs {
            entries.sort_by(|a, b| a.path().cmp(b.path()));
        } else {
            // Sort by depth descending, then by path.
            // This ensures we process children before parents even after sorting.
            entries.sort_by(|a, b| {
                let depth_a = a.depth();
                let depth_b = b.depth();
                depth_b.cmp(&depth_a).then_with(|| a.path().cmp(b.path()))
            });
        }

        let mut plan = Vec::new();
        for entry in entries {
            self.process_entry(entry.path(), |old, new| {
                plan.push((old.to_path_buf(), new.to_path_buf()));
            });
        }
        plan
    }

    fn filter_entry(&self, entry: &walkdir::DirEntry) -> bool {
        if !self.extensions.is_empty() {
            let match_ext = entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| self.extensions.contains(ext))
                .unwrap_or(false);
            if !match_ext {
                return false;
            }
        }

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
    }

    fn process_entry<F>(&self, path: &Path, mut f: F)
    where
        F: FnMut(&Path, &Path),
    {
        if let Some(old_file_name) = path.file_name().and_then(|n| n.to_str()) {
            let new_name = self.regex.replace_all(old_file_name, self.replacement);
            if old_file_name != new_name {
                let mut new_path = path.to_path_buf();
                let processed_name = self.process_dynamic_variables(&new_name, path);
                new_path.set_file_name(processed_name);
                f(path, &new_path);
            }
        }
    }

    /// Sequential version of `run`.
    pub fn run_seq<F>(&self, mut f: F)
    where
        F: FnMut(&Path, &Path),
    {
        let plan = self.generate_plan();
        for (old, new) in plan {
            f(&old, &new);
        }
    }

    /// Performs the bulk rename operation, notifying the provided `callback` of each outcome.
    ///
    /// Files are renamed in place. This operation is performed in parallel across multiple threads.
    pub fn execute(&self, callback: impl Callback) {
        let targets = Mutex::new(HashSet::new());
        self.run(|old_path, new_path| {
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

    fn process_dynamic_variables(&self, name: &str, path: &Path) -> String {
        let mut result = name.to_string();

        // Handle transformations {u:}, {l:}, {t:}
        result = self.process_transformations(result);

        // Handle {i} and {i:N}
        if result.contains("{i") {
            let i = self.counter.fetch_add(1, Ordering::SeqCst);
            let re_i = Regex::new(r"\{i(?::(\d+))?\}").unwrap();
            result = re_i
                .replace_all(&result, |caps: &regex::Captures| {
                    if let Some(padding) = caps.get(1) {
                        if let Ok(p) = padding.as_str().parse::<usize>() {
                            return format!("{:0>width$}", i, width = p);
                        }
                    }
                    i.to_string()
                })
                .to_string();
        }

        // Handle {date} and {date:FORMAT}
        if result.contains("{date") {
            let mtime = fs::metadata(path)
                .and_then(|m| m.modified())
                .map(DateTime::<Local>::from)
                .unwrap_or_else(|_| Local::now());

            let re_date = Regex::new(r"\{date(?::([^}]+))?\}").unwrap();
            result = re_date
                .replace_all(&result, |caps: &regex::Captures| {
                    if let Some(fmt) = caps.get(1) {
                        return mtime.format(fmt.as_str()).to_string();
                    }
                    mtime.format("%Y-%m-%d").to_string()
                })
                .to_string();
        }

        result
    }

    fn process_transformations(&self, mut s: String) -> String {
        let re = Regex::new(r"\{(u|upper|l|lower|t|title):([^{}]*)\}").unwrap();

        loop {
            let next = re
                .replace_all(&s, |caps: &regex::Captures| {
                    let transform = caps.get(1).unwrap().as_str();
                    let text = caps.get(2).unwrap().as_str();
                    match transform {
                        "u" | "upper" => text.to_uppercase(),
                        "l" | "lower" => text.to_lowercase(),
                        "t" | "title" => self.to_title_case(text),
                        _ => text.to_string(),
                    }
                })
                .to_string();

            if next == s {
                break;
            }
            s = next;
        }
        s
    }

    fn to_title_case(&self, s: &str) -> String {
        s.split_inclusive(|c: char| !c.is_alphanumeric())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => {
                        if f.is_alphanumeric() {
                            f.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                        } else {
                            f.to_string() + chars.as_str()
                        }
                    }
                }
            })
            .collect()
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
    fn test_to_title_case() {
        let dir = tempdir().unwrap();
        let bulk_rename = BulkRename::new(dir.path(), ".*", "replacement").unwrap();

        assert_eq!(bulk_rename.to_title_case("hello world"), "Hello World");
        assert_eq!(bulk_rename.to_title_case("HELLO WORLD"), "Hello World");
        assert_eq!(bulk_rename.to_title_case("hello_world"), "Hello_World");
        assert_eq!(bulk_rename.to_title_case("hello-world"), "Hello-World");
        assert_eq!(bulk_rename.to_title_case("123hello"), "123hello");
        assert_eq!(bulk_rename.to_title_case("a b c"), "A B C");
        assert_eq!(bulk_rename.to_title_case("test.txt"), "Test.Txt");
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

    #[test]
    fn test_bulk_rename_directories() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("sub");
        fs::create_dir(&sub_dir).unwrap();
        let file = sub_dir.join("file.txt");
        File::create(&file).unwrap();

        let bulk_rename = BulkRename::new(dir.path(), "sub", "SUB")
            .unwrap()
            .with_rename_files(false)
            .with_rename_dirs(true);

        let plan = bulk_rename.generate_plan();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].0, sub_dir);
        assert_eq!(plan[0].1, dir.path().join("SUB"));
    }

    #[test]
    fn test_bulk_rename_directories_recursive() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("sub");
        fs::create_dir(&sub_dir).unwrap();
        let sub_sub_dir = sub_dir.join("inner");
        fs::create_dir(&sub_sub_dir).unwrap();

        let bulk_rename = BulkRename::new(dir.path(), ".*", "{u:$0}")
            .unwrap()
            .with_rename_files(false)
            .with_rename_dirs(true);

        let plan = bulk_rename.generate_plan();
        // Should have "inner" -> "INNER" and "sub" -> "SUB"
        // Because of depth descending sort, "inner" should come first.
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].0, sub_sub_dir);
        assert_eq!(plan[0].1, sub_dir.join("INNER"));
        assert_eq!(plan[1].0, sub_dir);
        assert_eq!(plan[1].1, dir.path().join("SUB"));
    }
}
