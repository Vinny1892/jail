# jail

Lightweight sandbox to run AI agents and command-line tools with strong filesystem isolation, built on top of [bubblewrap (bwrap)](https://github.com/containers/bubblewrap).

It plays a role similar to a `virtualenv`, but focused on **isolating filesystem access**: processes only see the current project directory (and a few system directories as read-only), reducing the risk of dangerous commands such as `rm -rf /` or leaking your real `$HOME`.

> **Status**: experimental / WIP

## Requirements

- Linux with user namespace support.
- `bwrap` installed and available on `PATH`.
- Rust (stable) to build the tool from source.

## Installation

### From source

Clone the repository and build in release mode:

```bash
git clone <REPO-URL> jail
cd jail
# optional: adjust edition in Cargo.toml and flags in .cargo/config.toml
cargo build --release
```

The binary will be available at `target/release/jail`.

Optionally, install it into a directory on your `PATH` (e.g. `~/.local/bin`):

```bash
install -Dm755 target/release/jail ~/.local/bin/jail
```

> **Note about static binaries**: the project includes an example `.cargo/config.toml` with `rustflags = ["-C", "target-feature=+crt-static"]`. Depending on your toolchain/target, this may break compilation (especially with proc-macros). If you hit issues, comment out or remove that configuration, or configure an appropriate `*-musl` target.

## How it works

At a high level, `jail` does the following:

- Creates new user, PID, UTS and IPC namespaces (and optionally network).
- Mounts system directories as read-only: `/usr`, `/bin`, `/lib`, `/lib64`, `/etc`, `/opt`.
- Bind-mounts the **current directory** (`$PWD`) with write access and uses `--chdir` to it.
- Creates a sparse temporary `$HOME` under `/tmp` with a minimal structure (`.config`, `.local/share`, `.local/state/mise`).
- Bind-mounts your real `~/.config` as read-only into that sparse HOME.
- Provides a fake `/etc/hosts` with `localhost` and the hostname `ai-sandbox`.
- Optionally isolates the network using `--unshare-net`.
- Detects `mise` (if available), bind-mounts its directories as read-only and runs its initialization snippet before your command.

All of this is wired together via `std::process::Command` calling `bwrap` with the appropriate arguments.

## Usage

Basic syntax:

```bash
jail [OPTIONS] [CMD] [ARGS...]
```

If no command is provided, `jail` starts an interactive `bash` shell inside the jail.

### Examples

Interactive shell in the current project directory:

```bash
jail
```

Run a simple command inside the jail:

```bash
jail ls -la
```

Map additional directories as read-only:

```bash
jail --map /usr/share --map ../other-project bash
```

Also isolate the network (uses `--unshare-net` in `bwrap`):

```bash
jail --net bash
```

Combine options:

```bash
jail --net --map ../agent-data crush
```

Inside the shell, the prompt is customized:

```bash
(jail) /path/to/project $
```

indicating that you are inside the isolated environment.

## Flags and behavior

### `--map <PATH>`

Adds extra read-only bind mounts. Each `PATH` is mounted onto itself:

- `--map /some/path` → `--ro-bind /some/path /some/path`
- Relative paths are resolved against the project directory (`$PWD`).
- If the path does not exist, `jail` prints a warning to `stderr` and skips it.

### `--net`

When specified, adds `--unshare-net` to the `bwrap` invocation, creating an isolated network namespace.

- Without `--net`: the process inside the jail shares the host network stack.
- With `--net`: the network is isolated (useful to keep agents from making network calls, depending on how the namespace is configured).

### Target command `[CMD] [ARGS...]`

- If you provide a command, it is executed after the initialization snippet (Mise, if present).
- If you dont, the default is an interactive `bash` shell.

## Development

### Running tests

The project includes unit tests for the more self-contained pieces (`shell_escape`, helpers in `temp`, Mise initialization snippet):

```bash
cargo test
```

If you are using aggressive static-linking settings in `.cargo/config.toml` and hit `proc-macro` errors, comment/adjust those flags before running tests.

### Project structure

- `src/main.rs` – entry point, parses CLI and invokes `bwrap`.
- `src/cli.rs` – CLI definition using `clap`.
- `src/bwrap.rs` – builds `bwrap` arguments and runs the sandbox.
- `src/temp.rs` – helpers for sparse HOME and temporary `/etc/hosts`.
- `src/mise.rs` – detection and initialization for `mise` (if present).

## Contributing

Pull requests and issues are welcome. Suggestions around security hardening, mount presets and integrations with agent tools (Crush, Cursor, etc.) are especially appreciated.

Recommended workflow:

1. Open an issue describing the motivation (bugfix, feature, refactor, etc.).
2. Create a branch and implement the change with tests covering the behavior.
3. Run `cargo fmt`, `cargo clippy` (if configured) and `cargo test` before opening the PR.
4. Clearly describe in the PR the expected impact, security implications and how to test.

## Versioning

The project aims to follow **semantic versioning** (`MAJOR.MINOR.PATCH`):

- Breaking changes in CLI/behavior → bump `MAJOR`.
- Backwards-compatible features → bump `MINOR`.
- Bug fixes and internal adjustments → bump `PATCH`.

While the project is in its early stages, breaking changes may happen more frequently until a stable 1.0 is reached.

## Acknowledgments

This project is heavily inspired by the article **"AI Agents: Garantindo a Proteção do seu Sistema"** by [Fabio Akita (AkitaOnRails)](https://akitaonrails.com/), available at:

- https://akitaonrails.com/2026/01/10/ai-agents-garantindo-a-protecao-do-seu-sistema/

Many of the ideas and shell scripts from that article served as a direct basis for `jail`.

## License

This project is licensed under the terms of the **MIT License**.

See the `LICENSE` file for the full license text.
