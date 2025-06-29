//! Structurally diff two [`Snapshot`]

use std::{collections::BTreeMap, path::Path, sync::Arc};

use arcstr::ArcStr;
use snafu::Snafu;

use crate::snapshot::Snapshot;

#[derive(Debug, Clone)]
pub struct ChangeLog {
    added_repos: BTreeMap<String, ()>,
    removed_repos: BTreeMap<String, ()>,
    /// changes ordered by datetime
    log: Vec<Change>,
    /// changes per repo
    changes: BTreeMap<String, Change>
}

#[derive(Debug, Clone)]
pub struct Change {
    repo: ArcStr,
    title: ArcStr,
    description: ArcStr,
    author: ArcStr,
    datetime: (),
}

#[derive(Debug, Clone, Snafu)]
pub enum ChangeLogError {
    
}

type Result<T, E=ChangeLogError> = std::result::Result<T, E>;

impl ChangeLog {
    pub fn generate(orig: &Snapshot, target: &Snapshot, tree: impl AsRef<Path>) -> Result<Self> {
        todo!()
    }
}