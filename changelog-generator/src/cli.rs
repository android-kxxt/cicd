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
        help = "Use a handlebars template to render the changelog"
    )]
    pub template: Vec<String>,
    #[arg(
        short,
        long,
        help = "Output the changelog as JSON"
    )]
    pub json: Option<String>,
    #[arg(
        short,
        long,
        help = "Output the changelog in debug format"
    )]
    pub debug: Option<String>,
}
