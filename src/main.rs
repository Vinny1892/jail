mod bwrap;
mod cli;
mod mise;
mod temp;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;

fn main() {
    match real_main() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
}

fn real_main() -> Result<i32> {
    let cli = Cli::parse();
    let code = bwrap::run_bwrap(&cli)?;
    Ok(code)
}
