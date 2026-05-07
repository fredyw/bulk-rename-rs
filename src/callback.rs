use crate::models::RenameRecord;
use std::io;
use std::path::Path;

/// A callback for the bulk rename.
pub trait Callback {
    /// Called when a file rename operation was successful.
    fn on_ok(&mut self, old_path: &Path, new_path: &Path);

    /// Called when a file rename operation failed.
    fn on_error(&mut self, old_path: &Path, new_path: &Path, error: io::Error);

    /// Called when a rollback operation was successful.
    fn on_rollback_ok(&mut self, old_path: &Path, new_path: &Path);

    /// Called when a rollback operation failed.
    fn on_rollback_error(&mut self, old_path: &Path, new_path: &Path, error: io::Error);
}

impl<T: Callback + ?Sized> Callback for &mut T {
    fn on_ok(&mut self, old_path: &Path, new_path: &Path) {
        (**self).on_ok(old_path, new_path);
    }
    fn on_error(&mut self, old_path: &Path, new_path: &Path, error: io::Error) {
        (**self).on_error(old_path, new_path, error);
    }
    fn on_rollback_ok(&mut self, old_path: &Path, new_path: &Path) {
        (**self).on_rollback_ok(old_path, new_path);
    }
    fn on_rollback_error(&mut self, old_path: &Path, new_path: &Path, error: io::Error) {
        (**self).on_rollback_error(old_path, new_path, error);
    }
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
    fn on_ok(&mut self, _old_path: &Path, _new_path: &Path) {}

    fn on_error(&mut self, _old_path: &Path, _new_path: &Path, _error: io::Error) {}

    fn on_rollback_ok(&mut self, _old_path: &Path, _new_path: &Path) {}

    fn on_rollback_error(&mut self, _old_path: &Path, _new_path: &Path, _error: io::Error) {}
}

/// A `Callback` that records successful renames into a history.
pub struct HistoryCallback<'a, C: Callback> {
    inner: C,
    history: &'a mut Vec<RenameRecord>,
}

impl<'a, C: Callback> HistoryCallback<'a, C> {
    /// Creates a new `HistoryCallback`.
    pub fn new(inner: C, history: &'a mut Vec<RenameRecord>) -> Self {
        Self { inner, history }
    }
}

impl<'a, C: Callback> Callback for HistoryCallback<'a, C> {
    fn on_ok(&mut self, old_path: &Path, new_path: &Path) {
        self.inner.on_ok(old_path, new_path);
        self.history.push(RenameRecord {
            old_path: old_path.to_path_buf(),
            new_path: new_path.to_path_buf(),
        });
    }

    fn on_error(&mut self, old_path: &Path, new_path: &Path, error: io::Error) {
        self.inner.on_error(old_path, new_path, error);
    }

    fn on_rollback_ok(&mut self, old_path: &Path, new_path: &Path) {
        self.inner.on_rollback_ok(old_path, new_path);
    }

    fn on_rollback_error(&mut self, old_path: &Path, new_path: &Path, error: io::Error) {
        self.inner.on_rollback_error(old_path, new_path, error);
    }
}
