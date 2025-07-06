//! Structurally diff two [`Snapshot`]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Output,
    string::FromUtf8Error,
};

use arcstr::ArcStr;
use chrono::{DateTime, Utc};
use serde::Serialize;
use snafu::{OptionExt, ResultExt, Snafu};
use xshell::{Shell, cmd};

use crate::{
    repo_log::{RepoChangeLog, RepoChangelogError, generate_repo_changelog},
    snapshot::{CommitHash, RepoStatus, Snapshot},
};

#[derive(Debug, Clone, Serialize)]
pub struct NewRepoStatus {
    pub upstream: ArcStr,
    pub recent_changes: Vec<Change>,
    pub commit: CommitHash,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemovedRepoStatus {
    pub last_seen_commit: CommitHash,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangeLog {
    added_repos: BTreeMap<ArcStr, NewRepoStatus>,
    removed_repos: BTreeMap<ArcStr, RemovedRepoStatus>,
    /// changes ordered by datetime
    log: Vec<Change>,
    /// changes per repo
    changes: BTreeMap<ArcStr, RepoChangeLog>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ChangeKind {
    Merge,
    Normal,
}

#[derive(Debug, Clone, Serialize)]
pub struct Change {
    pub kind: ChangeKind,
    pub repo: ArcStr,
    pub title: ArcStr,
    pub description: ArcStr,
    pub author_name: ArcStr,
    pub author_email: ArcStr,
    pub datetime: DateTime<Utc>,
    pub change_id: Option<ArcStr>,
}

#[derive(Debug, Snafu)]
pub enum ChangeLogError {
    #[snafu(display("failed to open a command shell"))]
    ShellCreation { source: xshell::Error },
    #[snafu(display("command exec failed"))]
    CommandExecution { source: xshell::Error },
    #[snafu(display("command failed"))]
    CommandFailure {
        operation: &'static str,
        message: String,
    },
    #[snafu(display("failed to parse command output as UTF-8"))]
    InvalidEncoding { source: FromUtf8Error },
    #[snafu(display("no history found in {repo}"))]
    NoHistory { repo: ArcStr },
    #[snafu(display("failed to generate changelog for {repo}"))]
    SingleRepo {
        repo: ArcStr,
        source: RepoChangelogError,
    },
}

pub type Result<T, E = ChangeLogError> = std::result::Result<T, E>;

impl ChangeLog {
    pub fn generate(orig: &Snapshot, target: &Snapshot, tree: impl AsRef<Path>) -> Result<Self> {
        let orig_repos: BTreeSet<ArcStr> = orig.repos.keys().cloned().collect();
        let target_repos: BTreeSet<ArcStr> = orig.repos.keys().cloned().collect();
        let added = target_repos.difference(&orig_repos);
        let removed = orig_repos.difference(&target_repos);
        let common = orig_repos.intersection(&target_repos);
        let changed = common.filter(|r| orig.repos[r.as_str()] != target.repos[r.as_str()]);
        let mut changes = BTreeMap::new();
        let sync_stamp_branch = get_sync_stamp_branch(&tree)?;
        let mut added_repos = BTreeMap::new();
        let mut removed_repos = BTreeMap::new();

        // Get normal changelogs
        for repo in changed {
            let repo_changelog = generate_repo_changelog(
                &orig.repos[repo.as_str()],
                &target.repos[repo.as_str()],
                repo,
                tree.as_ref(),
            )
            .with_context(|_| SingleRepoSnafu { repo: repo.clone() })?;
            changes.insert(repo.to_owned(), repo_changelog);
        }
        // Generate for newly added repos
        for repo in added {
            let status = generate_new_repo_changelog(
                10,
                repo,
                &target.repos[repo.as_str()],
                tree.as_ref(),
                &sync_stamp_branch,
            )?;
            added_repos.insert(repo.clone(), status);
        }
        // Generate for removed repos
        for repo in removed {
            removed_repos.insert(
                repo.clone(),
                RemovedRepoStatus {
                    last_seen_commit: orig.repos[repo.as_str()].commit.clone(),
                },
            );
        }
        Ok(ChangeLog {
            added_repos,
            removed_repos,
            log: vec![],
            changes,
        })
    }
}

fn get_sync_stamp_branch(tree: &impl AsRef<Path>) -> Result<String> {
    let sh = Shell::new().context(ShellCreationSnafu)?;
    let top = tree.as_ref();
    let repo_info = output2string(
        cmd!(sh, "env -C {top} repo info")
            .ignore_status()
            .output()
            .context(CommandExecutionSnafu)?,
    )?;
    let manifest_branch = repo_info
        .lines()
        .filter_map(|s| s.strip_prefix("Manifest branch:").map(|rref| rref.trim()))
        .next()
        .with_context(|| CommandFailureSnafu {
            operation: "repo info",
            message: "Output does not contain manifest branch".to_string(),
        })?;
    let manifest_branch = manifest_branch
        .strip_prefix("refs/heads/")
        .unwrap_or(manifest_branch);

    Ok(format!("m/{manifest_branch}"))
}

fn generate_new_repo_changelog(
    max_recent_changes: usize,
    repo: &ArcStr,
    current: &RepoStatus,
    top: impl AsRef<Path>,
    sync_stamp_branch: &str,
) -> Result<NewRepoStatus> {
    let sh = Shell::new().context(ShellCreationSnafu)?;
    let repo = ArcStr::from(repo);
    let repo_path = top.as_ref().join(repo.as_str());
    let commit = current.commit.as_ref();
    // Get a start commit
    let limit = max_recent_changes.to_string();
    let recent_commits = output2string(
        cmd!(
            sh,
            "git -C {repo_path} rev-list --first-parent --reverse --max-count={limit} {commit}"
        )
        .output()
        .context(CommandExecutionSnafu)?,
    )?;
    let start = recent_commits
        .lines()
        .next()
        .with_context(|| NoHistorySnafu { repo: repo.clone() })?;
    // Get the canonical upstream url
    let upstream_ref = output2string(
        cmd!(
            sh,
            "git -C {repo_path} rev-parse --symbolic --abbrev-ref {sync_stamp_branch}"
        )
        .output()
        .context(CommandExecutionSnafu)?,
    )?;
    let upstream = upstream_ref
        .split_once('/')
        .with_context(|| CommandFailureSnafu {
            operation: "git rev-parse --symbolic --abbrev-ref",
            message: "the output does not contain a remote part",
        })?
        .0;
    let upstream = ArcStr::from(
        output2string(
            cmd!(sh, "git -C {repo_path} remote get-url {upstream}")
                .output()
                .context(CommandExecutionSnafu)?,
        )?
        .trim(),
    );

    // Generate log of recent changes
    let changelog = generate_repo_changelog(
        &RepoStatus {
            commit: CommitHash::try_new(start).unwrap(),
        },
        current,
        &repo,
        top,
    )
    .with_context(|_| SingleRepoSnafu { repo })?;
    Ok(NewRepoStatus {
        upstream,
        recent_changes: changelog.logs,
        commit: current.commit.clone(),
    })
}

pub(crate) fn output2string(output: Output) -> Result<String> {
    String::from_utf8(output.stdout).with_context(|_| InvalidEncodingSnafu)
}
