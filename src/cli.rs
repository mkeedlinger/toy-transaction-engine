use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    pub input_csv: PathBuf,

    /// Defaults to stdout if not set
    pub output_csv: Option<PathBuf>,
}
