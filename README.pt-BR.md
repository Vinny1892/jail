# ai-jail

Sandbox leve para executar agentes de IA e ferramentas de linha de comando com isolamento de sistema, baseado em [bubblewrap (bwrap)](https://github.com/containers/bubblewrap).

A ideia é semelhante a uma `virtualenv`, mas focada em **isolar acesso ao filesystem**: o processo só enxerga o diretório do projeto (e alguns diretórios de sistema em read-only), reduzindo o risco de comandos perigosos como `rm -rf /` ou vazamento da sua `$HOME` real.

> **Status**: experimental / WIP

## Requisitos

- Linux com suporte a user namespaces.
- `bwrap` instalado e acessível no `PATH`.
- Rust (stable) para compilar a ferramenta.

## Instalação

### A partir do código-fonte

Clone o repositório e compile em modo release:

```bash
git clone <URL-do-repo> ai-jail
cd ai-jail
# opcional: ajustar edition no Cargo.toml e flags em .cargo/config.toml
cargo build --release
```

O binário ficará em `target/release/ai-jail`.

Opcionalmente, instale em algum diretório do `PATH` (ex.: `~/.local/bin`):

```bash
install -Dm755 target/release/ai-jail ~/.local/bin/ai-jail
```

> **Nota sobre binário estático**: o projeto inclui um exemplo de `.cargo/config.toml` com `rustflags = ["-C", "target-feature=+crt-static"]`. Dependendo da sua toolchain/target, isso pode quebrar a compilação (especialmente com proc-macros). Se tiver problemas, comente ou remova essa configuração, ou configure um target `*-musl` adequado.

## Como funciona

Em alto nível, `ai-jail` faz o seguinte:

- Cria um novo namespace de usuário, PID, UTS, IPC (e opcionalmente de rede).
- Monta diretórios de sistema como read-only: `/usr`, `/bin`, `/lib`, `/lib64`, `/etc`, `/opt`.
- Liga (bind) o **diretório atual** (`$PWD`) com acesso de escrita e faz `--chdir` para ele.
- Cria um `$HOME` temporário esparso em `/tmp`, com estrutura mínima (`.config`, `.local/share`, `.local/state/mise`).
- Monta o `~/.config` real como read-only dentro desse HOME esparso.
- Cria um `/etc/hosts` fake com `localhost` e hostname `ai-sandbox`.
- Opcionalmente isola a rede usando `--unshare-net`.
- Detecta `mise` (se disponível), monta seus diretórios em read-only e roda o snippet de inicialização antes do comando.

Tudo isso é construído via `std::process::Command` chamando `bwrap` com os argumentos apropriados.

## Uso

Sintaxe básica:

```bash
ai-jail [OPÇÕES] [CMD] [ARGS...]
```

Se nenhum comando for fornecido, o padrão é abrir um `bash` interativo dentro do jail.

### Exemplos

Shell interativo no diretório do projeto atual:

```bash
ai-jail
```

Executar um comando simples dentro do jail:

```bash
ai-jail ls -la
```

Mapear diretórios adicionais como read-only:

```bash
ai-jail --map /usr/share --map ../outro-projeto bash
```

Isolar também a rede (usa `--unshare-net` no `bwrap`):

```bash
ai-jail --net bash
```

Combinar opções:

```bash
ai-jail --net --map ../dados-agente crush
```

Dentro do shell, o prompt será customizado:

```bash
(jail) /caminho/do/projeto $
```

indicando que você está dentro do ambiente isolado.

## Flags e comportamento

### `--map <PATH>`

Adiciona binds read-only extras. Cada `PATH` é montado em si mesmo:

- `--map /algum/path` → `--ro-bind /algum/path /algum/path`
- Caminhos relativos são resolvidos em relação ao diretório do projeto (`$PWD`).
- Se o path não existir, o ai-jail emite um aviso em `stderr` e ignora.

### `--net`

Quando especificado, adiciona `--unshare-net` à chamada do `bwrap`, criando um namespace de rede isolado.

- Sem `--net`: o processo dentro do jail usa a mesma pilha de rede do host.
- Com `--net`: a rede é isolada (útil para impedir que um agente faça chamadas de rede, dependendo de como o namespace for configurado).

### Comando alvo `[CMD] [ARGS...]`

- Se você passar um comando, ele é executado após o snippet de inicialização (Mise, se existir).
- Se não passar, o padrão é `bash` interativo.

## Desenvolvimento

### Rodando testes

O projeto inclui alguns testes unitários para as partes mais puras da lógica (`shell_escape`, helpers de `temp`, geração de snippet do `mise`):

```bash
cargo test
```

Se você estiver usando configurações de linkagem estática agressivas em `.cargo/config.toml` e tiver erros de `proc-macro`, comente/ajuste essas flags antes de rodar os testes.

### Estrutura do código

- `src/main.rs` – ponto de entrada, faz o parse da CLI e chama o `bwrap`.
- `src/cli.rs` – definição da linha de comando utilizando `clap`.
- `src/bwrap.rs` – construção dos argumentos do `bwrap` e execução do sandbox.
- `src/temp.rs` – helpers para HOME esparso e `/etc/hosts` temporário.
- `src/mise.rs` – detecção e inicialização de `mise` (se presente).

## Contribuindo

Pull requests e issues são bem-vindos. Sugestões de melhorias de segurança, presets de mounts e integrações com ferramentas de agentes (Crush, Cursor, etc.) são especialmente interessantes.

Recomendações:

1. Abra uma issue descrevendo a motivação da mudança (bugfix, feature, refactor, etc.).
2. Crie uma branch e implemente a mudança com testes cobrindo o comportamento.
3. Rode `cargo fmt`, `cargo clippy` (se estiver configurado) e `cargo test` antes de abrir o PR.
4. Descreva claramente no PR o impacto esperado, riscos de segurança e como testar.

## Versionamento

O projeto pretende seguir **versionamento semântico** (`MAJOR.MINOR.PATCH`):

- Quebra de compatibilidade de CLI/comportamento → bump em `MAJOR`.
- Novas features compatíveis → bump em `MINOR`.
- Correções de bug e ajustes internos → bump em `PATCH`.

Por estar em fase inicial, quebras de compatibilidade podem ocorrer com mais frequência até atingir uma 1.0 estável.

## Agradecimentos

Este projeto foi fortemente inspirado pelo artigo **"AI Agents: Garantindo a Proteção do seu Sistema"** do [Fabio Akita (AkitaOnRails)](https://akitaonrails.com/), disponível em:

- https://akitaonrails.com/2026/01/10/ai-agents-garantindo-a-protecao-do-seu-sistema/

Obrigado ao Akita por compartilhar scripts e ideias que serviram de base para o `ai-jail`.

## Licença

Este projeto é licenciado sob os termos da **MIT License**.

Veja o arquivo `LICENSE` para o texto completo da licença.
