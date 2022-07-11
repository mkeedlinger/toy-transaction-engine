//! Setup functions
//!
//! This module includes all functions required to
//! set up things like logging, metrics, etc.

use crate::prelude::*;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

pub fn setup() -> Result<()> {
    logger()?;
    Ok(())
}

fn logger() -> Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )?;
    Ok(())
}
