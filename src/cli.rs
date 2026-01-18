use std::path::PathBuf;

use clap::Parser;

/// ai-jail: execute comandos em um sandbox baseado em bubblewrap.
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Caminhos adicionais a montar como read-only dentro do jail.
    #[arg(long = "map", value_name = "PATH")]
    pub maps: Vec<PathBuf>,

    /// Isolar a rede usando --unshare-net (sem acesso direto à rede do host).
    #[arg(long = "net")]
    pub net: bool,

    /// Comando a executar dentro do jail (default: bash interativo).
    /// Ex.: ai-jail --map /algum/path ls -la
    #[arg(trailing_var_arg = true)]
    pub cmd: Vec<String>,
}
