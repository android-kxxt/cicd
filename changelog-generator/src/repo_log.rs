//! Generate changelog for a single repo

use std::{collections::HashSet, error::Error, path::Path, process::Output};

use arcstr::ArcStr;
use chrono::{DateTime, Utc};
use snafu::{OptionExt, ResultExt, Snafu};
use xshell::{Shell, cmd};

use crate::{
    changelog::{Change, ChangeKind},
    snapshot::RepoStatus,
};

#[derive(Debug, Snafu)]
pub enum RepoChangelogError {
    #[snafu(display("failed to open a command shell"))]
    ShellCreation { source: xshell::Error },
    #[snafu(display("command exec failed"))]
    CommandExecution { source: xshell::Error },
    #[snafu(whatever, display("git failed: {message}"))]
    Git {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn Error + 'static>>,
    },
    #[snafu(display("failed to parse commit {commit}: {reason}"))]
    ParseCommit {
        commit: String,
        raw: String,
        reason: &'static str,
    },
    #[snafu(display("failed to parse author {raw:?}: {reason}"))]
    ParseAuthor { raw: String, reason: &'static str },
    #[snafu(display("failed to parse date {raw:?}: {reason}"))]
    ParseDate { raw: String, reason: &'static str },
}

type Result<T, E = RepoChangelogError> = std::result::Result<T, E>;

struct RepoChangeLog {
    logs: Vec<Change>,
}

#[derive(Debug, Clone)]
struct ParsedCommit {
    author_name: ArcStr,
    author_email: ArcStr,
    commit_date: DateTime<Utc>,
    title: ArcStr,
    description: ArcStr,
    change_id: Option<ArcStr>,
}

pub fn parse_commit(commit: &str, details: String) -> Result<ParsedCommit> {
    // Date is UTC unix timestamp.                //
    // commit c91ae3e2afaee6a578b60fc31d0bd7e793cdf9aa (HEAD, m/lineage-22.2, github/main, github/HEAD)
    // Author:     kxxt <rsworktech@outlook.com>
    // AuthorDate: 1751211480
    // Commit:     kxxt <rsworktech@outlook.com>
    // CommitDate: 1751211480
    // <empty>
    //     Titles adf ad f dasfd
    //     overflowed title
    // <empty>
    //     Body
    // <empty>
    //     Trailers
    let lines = details.lines().skip(1);
    let mut author = None;
    let mut commit_date = None;
    let mut title = String::new();
    let mut description = String::new();
    let mut stage = 0;
    // headers
    for line in lines {
        if stage == 0 {
            if line == "" {
                // end of stage
                stage += 1;
                continue;
            }
            // parse headers
            let (key, value) = line.split_once(':').with_context(|| ParseCommitSnafu {
                commit: commit.to_string(),
                raw: details.to_string(),
                reason: "header does not contain key-value separator `:`",
            })?;
            let key = key.trim();
            let value = value.trim();
            match key {
                "Author" => author = Some(value),
                "CommitDate" => commit_date = Some(value),
                _ => continue,
            }
            continue;
        }

        let line = line
            .strip_prefix("    ")
            .with_context(|| ParseCommitSnafu {
                commit: commit.to_string(),
                raw: details.to_string(),
                reason: "Commit body line does not start with 4 spaces",
            })?;

        if stage == 1 {
            // parse title
            if line.is_empty() {
                if title.is_empty() {
                    return Err(RepoChangelogError::ParseCommit {
                        commit: commit.to_string(),
                        raw: details,
                        reason: "The commit does not have a title",
                    });
                } else {
                    stage += 1;
                    continue;
                }
            } else {
                if !title.is_empty() {
                    // join the title lines
                    title.push(' ');
                }
                title.push_str(line);
            }
        } else if stage == 2 {
            // parse body
            description.push_str(line.trim());
            description.push('\n');
        } else {
            unreachable!()
        }
    }
    let author = author.with_context(|| ParseCommitSnafu {
        commit: commit.to_string(),
        raw: details.to_string(),
        reason: "header does not contain Author field",
    })?;
    let (author_name, author_email) =
        author.rsplit_once(' ').with_context(|| ParseAuthorSnafu {
            raw: author.to_string(),
            reason: "Cannot split author into name and email",
        })?;
    let author_email = author_email
        .strip_prefix('<')
        .with_context(|| ParseAuthorSnafu {
            raw: author.to_string(),
            reason: "The email part does not begin with `<`",
        })?
        .strip_suffix('>')
        .with_context(|| ParseAuthorSnafu {
            raw: author.to_string(),
            reason: "The email part does not end with `>`",
        })?;
    let commit_date = commit_date.with_context(|| ParseCommitSnafu {
        commit: commit.to_string(),
        raw: details.to_string(),
        reason: "header does not contain CommitDate field",
    })?;
    let commit_date = commit_date
        .parse::<u32>()
        .ok()
        .with_context(|| ParseDateSnafu {
            raw: commit_date.to_string(),
            reason: "Failed to parse date as an unsigned integer. Did you use `--date=unix`?",
        })?;
    let commit_date =
        DateTime::<Utc>::from_timestamp(commit_date as i64, 0).with_context(|| ParseDateSnafu {
            raw: commit_date.to_string(),
            reason: "Date is out of range",
        })?;
    let title = ArcStr::from(title);
    // Let's see if it contains trailers
    let (description, trailers) =
        if let Some((description, trailers)) = description.rsplit_once("\n\n") {
            (description, Some(trailers))
        } else {
            (description.as_str(), None)
        };
    // Then get change id from the trailers
    let change_id = trailers.and_then(|s| {
        s.lines()
            .filter_map(|l| l.strip_prefix("Change-Id:"))
            .next()
            .map(|s| ArcStr::from(s.trim()))
    });
    Ok(ParsedCommit {
        author_name: ArcStr::from(author_name),
        author_email: ArcStr::from(author_email),
        commit_date,
        title,
        description: ArcStr::from(description),
        change_id,
    })
}

pub fn generate_repo_changelog(
    source: RepoStatus,
    target: RepoStatus,
    repo: &str,
    top: impl AsRef<Path>,
) -> Result<RepoChangeLog> {
    let sh = Shell::new().context(ShellCreationSnafu)?;
    let repo = ArcStr::from(repo);
    let repo_path = top.as_ref().join(repo.as_str());
    let source_commit = source.commit.as_ref();
    let target_commit = target.commit.as_ref();
    // We will do it in two pass,
    // in the first pass, we figure out which commits should be included into the changelog.

    // Get all merge commits, which are handled separately.
    let merge_commits = output2string(
        cmd!(
            sh,
            "git -C {repo_path} rev-list --min-parents=2 --count {source_commit}..{target_commit}"
        )
        .output()
        .context(CommandExecutionSnafu)?,
    )?;
    let merge_commits: HashSet<_> = merge_commits
        .lines()
        .into_iter()
        .map(|x| x.trim())
        .collect();

    let commits = output2string(
        cmd!(
            sh,
            "git -C {repo_path} rev-list --first-parent --count {source_commit}..{target_commit}"
        )
        .output()
        .context(CommandExecutionSnafu)?,
    )?;
    let commits: Vec<_> = commits.lines().into_iter().map(|x| x.trim()).collect();
    let mut logs = Vec::new();

    for commit in commits {
        let commit_details = output2string(
            cmd!(
                sh,
                "git -C {repo_path} show --format=fuller --date=unix --no-patch {commit}"
            )
            .output()
            .context(CommandExecutionSnafu)?,
        )?;
        if merge_commits.contains(commit) {
            logs.push(Change {
                kind: ChangeKind::Merge,
                repo: repo.clone(),
                title: todo!(),
                description: todo!(),
                author: todo!(),
                datetime: todo!(),
            });
        } else {
            logs.push(Change {
                kind: ChangeKind::Normal,
                repo: repo.clone(),
                title: todo!(),
                description: todo!(),
                author: todo!(),
                datetime: todo!(),
            });
        }
    }
    Ok(RepoChangeLog { logs })
}

fn output2string(output: Output) -> Result<String> {
    if !output.status.success() {
        return Err(RepoChangelogError::Git {
            message: format!(
                "git failed with {}, stderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
            source: None,
        });
    }
    String::from_utf8(output.stdout)
        .with_whatever_context(|_| format!("git output is not valid UTF-8"))
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::sync::Arc;

    #[test]
    fn test_parse_valid_commit() {
        let commit = "c91ae3e2afaee6a578b60fc31d0bd7e793cdf9aa";
        let details = r#"
Author:     kxxt <rsworktech@outlook.com>
AuthorDate: 1751211480
Commit:     kxxt <rsworktech@outlook.com>
CommitDate: 1751211480

    This is the commit title
    that continues in the next line
    
    This is the body.
    
    Change-Id: Iabc123xyz
"#
        .to_string();

        let result = parse_commit(commit, details).unwrap();

        assert_eq!(&*result.author_name, "kxxt");
        assert_eq!(&*result.author_email, "rsworktech@outlook.com");
        assert_eq!(
            result.commit_date,
            Utc.timestamp_opt(1751211480, 0).unwrap()
        );
        assert_eq!(
            &*result.title,
            "This is the commit title that continues in the next line"
        );
        assert_eq!(&*result.description, "This is the body.");
        assert_eq!(result.change_id.as_deref(), Some("Iabc123xyz"));
    }

    #[test]
    fn test_parse_commit_missing_title() {
        let commit = "abcdef";
        let details = r#"
Author:     John Doe <john@example.com>
CommitDate: 1751211480

    
    This is body without title.
"#
        .to_string();

        let err = parse_commit(commit, details.clone()).unwrap_err();
        assert!(format!("{}", err).contains("does not have a title"));
    }

    #[test]
    fn test_parse_commit_missing_author() {
        let commit = "abcdef";
        let details = r#"
CommitDate: 1751211480

    A valid title
"#
        .to_string();

        let err = parse_commit(commit, details.clone()).unwrap_err();
        assert!(format!("{}", err).contains("does not contain Author field"));
    }

    #[test]
    fn test_parse_commit_invalid_date() {
        let commit = "abcdef";
        let details = r#"
Author:     Someone <someone@example.com>
CommitDate: not_a_date

    A valid title
"#
        .to_string();

        let err = parse_commit(commit, details.clone()).unwrap_err();
        assert!(format!("{}", err).contains("Failed to parse date"));
    }

    #[test]
    fn test_parse_commit_no_trailers() {
        let commit = "abcdef";
        let details = r#"
Author:     Someone <someone@example.com>
CommitDate: 1751211480

    Simple title
    
    Body without trailer.
"#
        .to_string();

        let result = parse_commit(commit, details).unwrap();
        assert_eq!(result.change_id, None);
        assert_eq!(&*result.description, "Body without trailer.\n");
    }
}
