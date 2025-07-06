//! [`Snapshot`] captures the status of the entire tree,
//! which could be compared with each other.

use std::collections::BTreeMap;

use arcstr::ArcStr;
use nutype::nutype;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub repos: BTreeMap<ArcStr, RepoStatus>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoStatus {
    pub commit: CommitHash,
}

#[nutype(
    derive(
        Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Display, AsRef, Serialize
    ),
    validate(not_empty, regex = "[0-9a-f]{8, 40}")
)]
pub struct CommitHash(String);

type Result<T, E = SnapshotError> = std::result::Result<T, E>;

#[derive(Debug, Clone, Snafu)]
pub enum SnapshotError {
    #[snafu(display("The repo status line {input:?} cannot be parsed"))]
    InvalidRepoStatusInput { input: String },
    #[snafu(display("Repo {repo:?} is duplicated in input snapshot"))]
    DuplicatedRepo { repo: String },
    #[snafu(display("Invalid commit {commit:?} from {repo:?}"))]
    InvalidCommit {
        commit: String,
        repo: String,
        source: CommitHashError,
    },
}

impl Snapshot {
    /// Parse from string
    /// e.g.
    ///
    /// system/core: 413223ae32d8f
    pub fn parse(input: String) -> Result<Self> {
        let mut repos = BTreeMap::new();
        for line in input.lines() {
            let Some((repo, commit)) = line.rsplit_once(':') else {
                return Err(SnapshotError::InvalidRepoStatusInput {
                    input: line.to_string(),
                });
            };
            let repo = repo.trim();
            let commit = commit.trim();
            if repos.contains_key(repo) {
                return Err(SnapshotError::DuplicatedRepo {
                    repo: repo.to_string(),
                });
            }
            repos.insert(
                ArcStr::from(repo),
                RepoStatus {
                    commit: CommitHash::try_new(commit.to_string()).with_context(|_| {
                        InvalidCommitSnafu {
                            commit: commit.to_string(),
                            repo: repo.to_string(),
                        }
                    })?,
                },
            );
        }
        Ok(Self { repos })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_snapshot() {
        let input = "system/core: 413223ae32d8f\nexternal/lib: abcd1234ef567890".to_string();
        let snapshot = Snapshot::parse(input).expect("Should parse successfully");

        assert_eq!(snapshot.repos.len(), 2);
        assert!(snapshot.repos.contains_key("system/core"));
        assert!(snapshot.repos.contains_key("external/lib"));
        assert_eq!(
            snapshot.repos["system/core"].commit,
            CommitHash::try_new("413223ae32d8f".to_string()).unwrap()
        );
    }

    #[test]
    fn test_parse_invalid_line_format() {
        let input = "system/core 413223ae32d8f".to_string(); // Missing colon
        let err = Snapshot::parse(input).unwrap_err();

        match err {
            SnapshotError::InvalidRepoStatusInput { input: e } => {
                assert_eq!(e, "system/core 413223ae32d8f")
            }
            _ => panic!("Expected InvalidRepoStatusInput error"),
        }
    }

    #[test]
    fn test_parse_duplicate_repo() {
        let input = "system/core: 413223ae32d8f\nsystem/core: abcd1234ef".to_string();
        let err = Snapshot::parse(input).unwrap_err();

        match err {
            SnapshotError::DuplicatedRepo { repo } => assert_eq!(repo, "system/core"),
            _ => panic!("Expected DuplicatedRepo error"),
        }
    }

    #[test]
    fn test_parse_invalid_commit_hash() {
        let input = "system/core: invalid_commit".to_string();
        let err = Snapshot::parse(input).unwrap_err();

        match err {
            SnapshotError::InvalidCommit { commit, repo, .. } => {
                assert_eq!(commit, "invalid_commit");
                assert_eq!(repo, "system/core");
            }
            _ => panic!("Expected InvalidCommit error"),
        }
    }

    #[test]
    fn test_parse_trims_whitespace() {
        let input = " system/core :  413223ae32d8f  ".to_string();
        let snapshot = Snapshot::parse(input).expect("Should parse successfully");
        assert!(snapshot.repos.contains_key("system/core"));
    }
}
