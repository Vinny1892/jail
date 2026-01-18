use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Configuração detectada do Mise (se disponível).
#[derive(Debug, Clone)]
pub struct MiseConfig {
    pub bin_path: PathBuf,
    pub data_dir: Option<PathBuf>,
}

/// Tenta detectar o binário `mise` e seu diretório de dados padrão.
pub fn detect_mise() -> Result<Option<MiseConfig>> {
    // Primeiro tenta localizar o binário via PATH.
    let bin_path = match which::which("mise") {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };

// Typical data directory: ~/.local/share/mise
    let home = env::var("HOME").context("HOME not set in environment")?;
    let home_path = Path::new(&home);
    let data_dir = {
        let candidate = home_path.join(".local/share/mise");
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    };

    Ok(Some(MiseConfig { bin_path, data_dir }))
}

/// Retorna o snippet de shell para inicializar o Mise dentro do bash.
pub fn mise_init_snippet(mise: &MiseConfig) -> String {
    let bin = mise.bin_path.to_string_lossy();
    format!(
        "{bin} trust >/dev/null 2>&1 || true; \
         eval \"$({bin} activate bash)\"; \
         eval \"$({bin} env)\";",
    )
}

/// Snippet padrão usado quando não há Mise disponível.
pub fn default_init_snippet() -> String {
    "true".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_init_snippet_is_true() {
        assert_eq!(default_init_snippet(), "true");
    }

    #[test]
    fn mise_init_snippet_contains_expected_commands() {
        let cfg = MiseConfig {
            bin_path: PathBuf::from("/usr/bin/mise"),
            data_dir: None,
        };
        let snippet = mise_init_snippet(&cfg);
        assert!(snippet.contains("trust"));
        assert!(snippet.contains("activate bash"));
        assert!(snippet.contains("env"));
    }
}
