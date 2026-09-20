

use std::path::PathBuf;

use clap::Parser;
/// Cli for Hive
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Program to run
    pub program: PathBuf,
}