use std::fmt;
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
