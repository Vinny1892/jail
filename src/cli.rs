use std::path::PathBuf;

use clap::Parser;

/// ai-jail: run commands in a bubblewrap-based sandbox.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Additional paths to mount as read-only inside the jail.
    #[arg(long = "map", value_name = "PATH")]
    pub maps: Vec<PathBuf>,

    /// Isolate network using --unshare-net (no direct access to host).
    #[arg(long = "net")]
    pub net: bool,

    /// Command to execute inside the jail (default: interactive bash).
    /// Example: ai-jail --map /some/path ls -la
    #[arg(trailing_var_arg = true)]
    pub cmd: Vec<String>,
}
