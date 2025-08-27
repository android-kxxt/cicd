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
    io::{BufReader, BufWriter, Read, Write},
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
    if cli.debug.is_none() && cli.json.is_none() && cli.template.is_empty() {
        bail!(
            "Please choose at least one output format using --json=<OUTPUT>/--debug=<OUTPUT>/--template=<OUTPUT>@<PATH_TO_HANDLEBARS_TEMPALATE>"
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
        bail!("--from={} does not exist", cli.from)
    };
    let to_fd = if std::fs::exists(format!("/proc/self/fd/{}", cli.to))
        .context("failed to check existence of --to-fd")?
    {
        unsafe { File::from_raw_fd(cli.to as RawFd) }
    } else {
        bail!("--to={} does not exist", cli.to)
    };
    let mut orig = String::new();
    let mut target = String::new();
    BufReader::new(from_fd).read_to_string(&mut orig)?;
    BufReader::new(to_fd).read_to_string(&mut target)?;
    let orig = Snapshot::parse(orig)?;
    let target = Snapshot::parse(target)?;
    let changelog = ChangeLog::generate(&orig, &target, cli.tree)?;
    if let Some(output) = cli.debug {
        std::fs::write(output, format!("{changelog:#?}"))?;
    }
    if let Some(output) = cli.json {
        let mut writer = BufWriter::new(File::create(output)?);
        serde_json::to_writer_pretty(&mut writer, &changelog)?;
        writer.flush()?;
    }
    for arg in cli.template {
        let Some((output, template)) = arg.split_once('@') else {
            bail!(
                "--template={arg} should specify output path and template path like -t=output@template"
            )
        };
        let template = std::fs::read_to_string(template)?;
        let formatted = template::format_changelog(template, &changelog)?;
        std::fs::write(output, formatted)?;
    }
    Ok(())
}
