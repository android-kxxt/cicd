use std::path::PathBuf;

use palc::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(long, help = "The original tree status")]
    pub from: String,
    #[arg(long, help = "The target tree status. (Default: current tree)")]
    pub to: Option<String>,
    pub tree: PathBuf,
}
