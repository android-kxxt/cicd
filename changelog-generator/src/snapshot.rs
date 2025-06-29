//! [`Snapshot`] captures the status of the entire tree,
//! which could be compared with each other.

use std::{collections::BTreeMap, path::Path};

use snafu::Snafu;

#[derive(Debug, Clone)]
pub struct Snapshot {
    repos: BTreeMap<String, RepoStatus>
}

#[derive(Debug, Clone)]
pub struct RepoStatus {
    commit: String
}

type Result<T, E=SnapshotError> = std::result::Result<T, E>;

#[derive(Debug, Clone, Snafu)]
pub enum SnapshotError {

}

impl Snapshot {
    /// Capture a snapshot of a tree
    pub fn capture(tree: impl AsRef<Path>) -> Result<Self> {
        todo!()
    }

    /// Parse from string
    /// e.g.
    /// 
    /// system/core: 413223ae32d8f
    pub fn parse(input: String) -> Result<Self> {
        todo!()
    }
}