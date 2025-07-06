//! Changelog Generator
//!
//! This generator generates changelog for superprojects managed by `repo`.
//! To achieve that, we scrape the git log of each git repo from its original commit to
//! the target commit.
//!
//! This generator needs to run inside a repo checkout as it needs to invoke git for getting
//! all the details. (TODO: maybe use a library instead of shelling out)
//!
//! Optionally, we support excluding some repos and explicitly include some repos to create
//! for example a device-specific changelog for AOSP builds.
//!
//! And we should also report updates in manifests repo and local_manifests
//! (provided that it is a git repo)

use std::{
    fs::File,
    io::{BufReader, Read, stdout},
    os::fd::{FromRawFd, RawFd},
};

use color_eyre::eyre::{Context, bail};
use palc::Parser;

use crate::{changelog::ChangeLog, cli::Cli, snapshot::Snapshot};

mod changelog;
mod cli;
mod repo_log;
mod snapshot;
mod template;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    if !cli.debug && !cli.json && cli.template.is_none() {
        bail!(
            "Please choose one output format using --json/--debug/--template=<PATH_TO_HANDLEBARS_TEMPALATE>"
        )
    }
    for fd in [cli.from, cli.to] {
        if fd == 2 || fd == 1 {
            bail!("Cannot use stdout/stderr for that!")
        }
    }
    let from_fd = if std::fs::exists(format!("/proc/self/fd/{}", cli.from))
        .context("failed to check existence of --from-fd")?
    {
        unsafe { File::from_raw_fd(cli.from as RawFd) }
    } else {
        bail!("--from-fd={} does not exist", cli.from)
    };
    let to_fd = if std::fs::exists(format!("/proc/self/fd/{}", cli.to))
        .context("failed to check existence of --to-fd")?
    {
        unsafe { File::from_raw_fd(cli.to as RawFd) }
    } else {
        bail!("--to-fd={} does not exist", cli.to)
    };
    let mut orig = String::new();
    let mut target = String::new();
    BufReader::new(from_fd).read_to_string(&mut orig)?;
    BufReader::new(to_fd).read_to_string(&mut target)?;
    let orig = Snapshot::parse(orig)?;
    let target = Snapshot::parse(target)?;
    let changelog = ChangeLog::generate(&orig, &target, cli.tree)?;
    if cli.debug {
        println!("{changelog:#?}");
    } else if cli.json {
        serde_json::to_writer_pretty(stdout().lock(), &changelog)?;
    } else if let Some(template) = cli.template {
        let template = std::fs::read_to_string(template)?;
        let formatted = template::format_changelog(template, &changelog)?;
        println!("{formatted}")
    }
    Ok(())
}
