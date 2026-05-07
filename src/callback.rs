use std::io;
use std::path::Path;

/// A callback for the bulk rename.
pub trait Callback: Sync + Send {
    /// Called when a file rename operation was successful.
    fn on_ok(&self, old_path: &Path, new_path: &Path);

    /// Called when a file rename operation failed.
    fn on_error(&self, old_path: &Path, new_path: &Path, error: io::Error);
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
}
