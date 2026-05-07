//! `bulk-rename-rs` is a command-line tool and library for bulk renaming files using regular expressions, Python scripting, and dynamic transformations.

pub mod callback;
pub mod engine;
pub mod error;
pub mod models;

pub use callback::{Callback, HistoryCallback, NoOpCallback};
pub use engine::BulkRename;
pub use error::Error;
pub use models::{
    CollisionStrategy, RenameHistory, RenameRecord, SymlinkStrategy, TransactionStrategy,
};
