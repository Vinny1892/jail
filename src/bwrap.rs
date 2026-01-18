use std::env;
use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::cli::Cli;
use crate::mise;
use crate::temp;

/// Constrói os argumentos do bwrap e executa o sandbox.
pub fn run_bwrap(cli: &Cli) -> Result<i32> {
    let project_dir = env::current_dir().context("Falha ao obter diretório atual (projeto)")?;
    let home = env::var("HOME").context("Variável de ambiente HOME não definida")?;
    let home_path = Path::new(&home);

    let (_sparse_home_dir_handle, sparse_home_path) =
        temp::create_sparse_home().context("Falha ao criar HOME esparso temporário")?;
    let hosts_file = temp::create_hosts_file("ai-sandbox")
        .context("Falha ao criar arquivo temporário de hosts")?;

    let real_home_config = temp::real_home_config(home_path);

    // Detecta Mise (se existir).
    let mise_cfg = match mise::detect_mise() {
        Ok(Some(cfg)) => {
            eprintln!("Mise detectado em {}", cfg.bin_path.display());
            Some(cfg)
        }
        Ok(None) => None,
        Err(err) => {
            eprintln!("Aviso: falha ao detectar Mise: {err}");
            None
        }
    };

    // Snippet de inicialização (Mise ou default).
    let init_snippet = match &mise_cfg {
        Some(cfg) => mise::mise_init_snippet(cfg),
        None => mise::default_init_snippet(),
    };

    let cmd_string = build_shell_command(&init_snippet, &cli.cmd);

    let mut args: Vec<OsString> = Vec::new();

    // Namespaces básicos.
    args.push("--unshare-user".into());
    args.push("--unshare-pid".into());
    args.push("--unshare-uts".into());
    args.push("--unshare-ipc".into());
    // Opcionalmente poderíamos tentar --unshare-cgroup aqui.

    if cli.net {
        // Isola a rede se solicitado.
        args.push("--unshare-net".into());
    }

    // Comportamento de vida do processo.
    args.push("--die-with-parent".into());
    args.push("--hostname".into());
    args.push("ai-sandbox".into());

    // /dev, /proc, tmpfs.
    args.push("--dev".into());
    args.push("/dev".into());
    args.push("--proc".into());
    args.push("/proc".into());

    args.push("--tmpfs".into());
    args.push("/tmp".into());
    args.push("--tmpfs".into());
    args.push("/run".into());

    // Diretórios de sistema read-only.
    for sys_dir in ["/usr", "/bin", "/lib", "/lib64", "/etc", "/opt"] {
        if Path::new(sys_dir).exists() {
            args.push("--ro-bind".into());
            args.push(sys_dir.into());
            args.push(sys_dir.into());
        }
    }

    // /etc/hosts fake.
    args.push("--ro-bind".into());
    args.push(hosts_file.path().as_os_str().into());
    args.push("/etc/hosts".into());

    // Diretório de projeto com escrita.
    args.push("--bind".into());
    args.push(project_dir.as_os_str().into());
    args.push(project_dir.as_os_str().into());

    args.push("--chdir".into());
    args.push(project_dir.as_os_str().into());

    // HOME esparso.
    args.push("--bind".into());
    args.push(sparse_home_path.as_os_str().into());
    args.push(sparse_home_path.as_os_str().into());

    // HOME dentro do sandbox.
    args.push("--setenv".into());
    args.push("HOME".into());
    args.push(sparse_home_path.as_os_str().into());

    // Monta ~/.config real como read-only dentro do HOME esparso, se existir.
    if let Some(cfg) = real_home_config {
        let target = sparse_home_path.join(".config");
        args.push("--ro-bind".into());
        args.push(cfg.as_os_str().into());
        args.push(target.as_os_str().into());
    }

    // Monta Mise (binário e data dir) como read-only, se detectado.
    if let Some(cfg) = &mise_cfg {
        // Binário.
        args.push("--ro-bind".into());
        args.push(cfg.bin_path.as_os_str().into());
        args.push(cfg.bin_path.as_os_str().into());

        // Diretório de dados.
        if let Some(data_dir) = &cfg.data_dir {
            args.push("--ro-bind".into());
            args.push(data_dir.as_os_str().into());
            args.push(data_dir.as_os_str().into());
        }
    }

    // Binds extras via --map PATH (sempre read-only).
    for path in &cli.maps {
        if !path.exists() {
            eprintln!("Aviso: caminho passado em --map não existe, ignorando: {}", path.display());
            continue;
        }
        let abs = if path.is_absolute() {
            path.clone()
        } else {
            project_dir.join(path)
        };
        args.push("--ro-bind".into());
        args.push(abs.as_os_str().into());
        args.push(abs.as_os_str().into());
    }

    // Prompt custom dentro do jail.
    args.push("--setenv".into());
    args.push("PS1".into());
    args.push("(jail) \\w \\$ ".into());

    // Comando final a executar.
    args.push("bash".into());
    args.push("-c".into());
    args.push(OsString::from(cmd_string));

    eprintln!("Jail ativo em: {}", project_dir.display());
    if cli.net {
        eprintln!("Rede isolada (--net) ativada.");
    }

    let status = Command::new("bwrap")
        .args(&args)
        .status()
        .map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => anyhow::Error::new(err)
                .context("bwrap não encontrado no PATH. Instale o bubblewrap (bwrap)."),
            _ => anyhow::Error::new(err),
        })?;

    if let Some(code) = status.code() {
        Ok(code)
    } else {
        bail!("Processo encerrado por sinal");
    }
}

fn build_shell_command(init_snippet: &str, cmd: &[String]) -> String {
    if cmd.is_empty() {
        format!("{init_snippet}; bash")
    } else {
        let escaped = cmd
            .iter()
            .map(|s| shell_escape(s))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{init_snippet}; {escaped}")
    }
}

fn shell_escape(arg: &str) -> String {
    if arg.is_empty() {
        "''".to_string()
    } else if !arg.contains(|c: char| c.is_whitespace() || "'\"$`!{}[]*?\\".contains(c)) {
        arg.to_string()
    } else {
        let mut s = String::from("'");
        for ch in arg.chars() {
            if ch == '\'' {
                s.push_str("'\\''");
            } else {
                s.push(ch);
            }
        }
        s.push('\'');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_escape_keeps_simple_word() {
        assert_eq!(shell_escape("ls"), "ls");
    }

    #[test]
    fn shell_escape_quotes_with_spaces() {
        assert_eq!(shell_escape("ls -la"), "'ls -la'");
    }

    #[test]
    fn shell_escape_handles_single_quote() {
        let out = shell_escape("foo'bar");
        // Representação clássica: 'foo'\''bar'
        assert_eq!(out, "'foo'\\''bar'");
    }

    #[test]
    fn shell_escape_empty_string() {
        assert_eq!(shell_escape(""), "''");
    }

    #[test]
    fn build_shell_command_no_args_defaults_to_bash() {
        let init = "init_snippet";
        let cmd = Vec::<String>::new();
        let built = build_shell_command(init, &cmd);
        assert_eq!(built, "init_snippet; bash");
    }

    #[test]
    fn build_shell_command_with_args_joins_and_escapes() {
        let init = "init";
        let cmd = vec!["ls".to_string(), "-la".to_string(), "some dir".to_string()];
        let built = build_shell_command(init, &cmd);
        assert_eq!(built, "init; ls -la 'some dir'");
    }
}
