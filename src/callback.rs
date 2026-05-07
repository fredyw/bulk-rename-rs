use crate::models::RenameRecord;
use std::io;
use std::path::Path;
use std::sync::Mutex;

/// A callback for the bulk rename.
pub trait Callback: Sync + Send {
    /// Called when a file rename operation was successful.
    fn on_ok(&self, old_path: &Path, new_path: &Path);

    /// Called when a file rename operation failed.
    fn on_error(&self, old_path: &Path, new_path: &Path, error: io::Error);

    /// Called when a rollback operation was successful.
    fn on_rollback_ok(&self, old_path: &Path, new_path: &Path);

    /// Called when a rollback operation failed.
    fn on_rollback_error(&self, old_path: &Path, new_path: &Path, error: io::Error);
}

/// A no-op `Callback`.
#[derive(Default)]
pub struct NoOpCallback {}

impl NoOpCallback {
    /// Creates a new no-op `Callback`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Callback for NoOpCallback {
    fn on_ok(&self, _old_path: &Path, _new_path: &Path) {}

    fn on_error(&self, _old_path: &Path, _new_path: &Path, _error: io::Error) {}

    fn on_rollback_ok(&self, _old_path: &Path, _new_path: &Path) {}

    fn on_rollback_error(&self, _old_path: &Path, _new_path: &Path, _error: io::Error) {}
}

/// A `Callback` that records successful renames into a history.
pub struct HistoryCallback<'a, C: Callback> {
    inner: C,
    history: &'a Mutex<Vec<RenameRecord>>,
}

impl<'a, C: Callback> HistoryCallback<'a, C> {
    /// Creates a new `HistoryCallback`.
    pub fn new(inner: C, history: &'a Mutex<Vec<RenameRecord>>) -> Self {
        Self { inner, history }
    }
}

impl<'a, C: Callback> Callback for HistoryCallback<'a, C> {
    fn on_ok(&self, old_path: &Path, new_path: &Path) {
        self.inner.on_ok(old_path, new_path);
        let mut history = self.history.lock().unwrap();
        history.push(RenameRecord {
            old_path: old_path.to_path_buf(),
            new_path: new_path.to_path_buf(),
        });
    }

    fn on_error(&self, old_path: &Path, new_path: &Path, error: io::Error) {
        self.inner.on_error(old_path, new_path, error);
    }

    fn on_rollback_ok(&self, old_path: &Path, new_path: &Path) {
        self.inner.on_rollback_ok(old_path, new_path);
    }

    fn on_rollback_error(&self, old_path: &Path, new_path: &Path, error: io::Error) {
        self.inner.on_rollback_error(old_path, new_path, error);
    }
}
