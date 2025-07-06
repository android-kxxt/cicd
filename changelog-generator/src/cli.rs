use std::path::PathBuf;

use palc::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(long, help = "The original tree status fd")]
    pub from: u32,
    #[arg(long, help = "The target tree status fd")]
    pub to: u32,
    pub tree: PathBuf,
    #[arg(
        short,
        long,
        conflicts_with = "json",
        conflicts_with = "debug",
        help = "Use a handlebars template to render the changelog"
    )]
    pub template: Option<PathBuf>,
    #[arg(
        short,
        long,
        conflicts_with = "debug",
        conflicts_with = "template",
        help = "Output the changelog as JSON"
    )]
    pub json: bool,
    #[arg(
        short,
        long,
        conflicts_with = "json",
        conflicts_with = "template",
        help = "Output the changelog in debug format"
    )]
    pub debug: bool,
}
