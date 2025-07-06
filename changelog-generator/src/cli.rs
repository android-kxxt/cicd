use std::path::PathBuf;

use palc::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(long, help = "The original tree status fd")]
    pub from: u32,
    #[arg(long, help = "The target tree status fd")]
    pub to: u32,
    pub tree: PathBuf,
}
