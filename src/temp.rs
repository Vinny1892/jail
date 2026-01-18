use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tempfile::{NamedTempFile, TempDir};

/// Cria um diretório HOME esparso em /tmp com a estrutura mínima esperada.
pub fn create_sparse_home() -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new_in("/tmp")?;
    let home_path = temp_dir.path().to_path_buf();

    // Estrutura básica de diretórios esperados por várias ferramentas.
    for rel in [
        ".config",
        ".local/share",
        ".local/state",
        ".local/state/mise",
    ] {
        let dir = home_path.join(rel);
        fs::create_dir_all(&dir)?;
    }

    Ok((temp_dir, home_path))
}

/// Cria um arquivo /etc/hosts temporário com localhost e o hostname desejado.
pub fn create_hosts_file(hostname: &str) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "127.0.0.1 localhost {}", hostname)?;
    writeln!(file, "::1       localhost {}", hostname)?;
    Ok(file)
}

/// Se existir, retorna o caminho para ~/.config do usuário real.
pub fn real_home_config(home: &Path) -> Option<PathBuf> {
    let cfg = home.join(".config");
    if cfg.exists() {
        Some(cfg)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sparse_home_creates_expected_structure() {
        let (_handle, home) = create_sparse_home().expect("sparse home");
        assert!(home.join(".config").is_dir());
        assert!(home.join(".local/share").is_dir());
        assert!(home.join(".local/state/mise").is_dir());
    }

    #[test]
    fn create_hosts_file_contains_hostname() {
        let hostname = "ai-sandbox-test";
        let file = create_hosts_file(hostname).expect("hosts file");
        let contents = std::fs::read_to_string(file.path()).expect("read hosts");
        assert!(contents.contains(hostname));
        assert!(contents.contains("127.0.0.1"));
        assert!(contents.contains("::1"));
    }

    #[test]
    fn real_home_config_returns_none_for_missing_config() {
        let tmp = TempDir::new().expect("tmp");
        assert!(real_home_config(tmp.path()).is_none());
    }

    #[test]
    fn real_home_config_returns_some_when_config_exists() {
        let tmp = TempDir::new().expect("tmp");
        let cfg = tmp.path().join(".config");
        std::fs::create_dir_all(&cfg).expect("mkdir");
        let res = real_home_config(tmp.path());
        assert_eq!(res.as_deref(), Some(cfg.as_path()));
    }
}
