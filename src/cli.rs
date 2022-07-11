use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    /// An input CSV file
    #[clap(long_help = INPUT_LONG_ABOUT)]
    pub input_csv: PathBuf,

    /// Defaults to stdout if not set
    pub output_csv: Option<PathBuf>,
}

const INPUT_LONG_ABOUT: &str = r#"
An input CSV file

It is expected to be in the following example form:

type,         client,   tx,   amount
deposit,           1,    1,      1.0
deposit,           2,    2,      2.0
deposit,           1,    3,      2.0
withdrawal,        1,    4,      1.5
withdrawal,        2,    5,      3.0
"#;
