use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

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

/// Strategies for handling symbolic links.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SymlinkStrategy {
    /// Ignore symbolic links.
    #[default]
    Ignore,
    /// Rename the symbolic link itself.
    Rename,
    /// Follow the symbolic link and rename the target.
    Follow,
}

impl FromStr for SymlinkStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ignore" => Ok(SymlinkStrategy::Ignore),
            "rename" => Ok(SymlinkStrategy::Rename),
            "follow" => Ok(SymlinkStrategy::Follow),
            _ => Err(format!(
                "invalid symlink strategy: {}. Valid values are: ignore, rename, follow",
                s
            )),
        }
    }
}

impl fmt::Display for SymlinkStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SymlinkStrategy::Ignore => "ignore",
            SymlinkStrategy::Rename => "rename",
            SymlinkStrategy::Follow => "follow",
        };
        write!(f, "{}", s)
    }
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

/// A record of a single file rename operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenameRecord {
    /// The original path of the file.
    pub old_path: PathBuf,
    /// The new path of the file.
    pub new_path: PathBuf,
}

/// A collection of rename records representing a history of operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenameHistory {
    /// The list of rename records.
    pub records: Vec<RenameRecord>,
}

impl RenameHistory {
    /// Creates a new, empty `RenameHistory`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a record to the history.
    pub fn add(&mut self, old_path: PathBuf, new_path: PathBuf) {
        self.records.push(RenameRecord { old_path, new_path });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_strategy_from_str() {
        assert_eq!(
            CollisionStrategy::from_str("skip").unwrap(),
            CollisionStrategy::Skip
        );
        assert_eq!(
            CollisionStrategy::from_str("OVERWRITE").unwrap(),
            CollisionStrategy::Overwrite
        );
        assert_eq!(
            CollisionStrategy::from_str("Suffix").unwrap(),
            CollisionStrategy::Suffix
        );
        assert!(CollisionStrategy::from_str("invalid").is_err());
    }

    #[test]
    fn test_collision_strategy_display() {
        assert_eq!(CollisionStrategy::Skip.to_string(), "skip");
        assert_eq!(CollisionStrategy::Overwrite.to_string(), "overwrite");
        assert_eq!(CollisionStrategy::Suffix.to_string(), "suffix");
    }

    #[test]
    fn test_collision_strategy_default() {
        assert_eq!(CollisionStrategy::default(), CollisionStrategy::Skip);
    }

    #[test]
    fn test_symlink_strategy_from_str() {
        assert_eq!(
            SymlinkStrategy::from_str("ignore").unwrap(),
            SymlinkStrategy::Ignore
        );
        assert_eq!(
            SymlinkStrategy::from_str("RENAME").unwrap(),
            SymlinkStrategy::Rename
        );
        assert_eq!(
            SymlinkStrategy::from_str("Follow").unwrap(),
            SymlinkStrategy::Follow
        );
        assert!(SymlinkStrategy::from_str("invalid").is_err());
    }

    #[test]
    fn test_symlink_strategy_display() {
        assert_eq!(SymlinkStrategy::Ignore.to_string(), "ignore");
        assert_eq!(SymlinkStrategy::Rename.to_string(), "rename");
        assert_eq!(SymlinkStrategy::Follow.to_string(), "follow");
    }

    #[test]
    fn test_symlink_strategy_default() {
        assert_eq!(SymlinkStrategy::default(), SymlinkStrategy::Ignore);
    }
}
