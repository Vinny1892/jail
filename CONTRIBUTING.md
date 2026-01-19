# Contributing to jail

Thanks for your interest in contributing to **jail**! This document explains how to propose changes, report issues, and submit pull requests.

## Code of conduct

By participating in this project, you agree to be respectful and constructive. Harassment, personal attacks, or any form of discrimination are not tolerated.

## How to report bugs

1. Search existing issues to see if the problem is already reported.
2. If not, open a new issue and include:
   - A clear description of the bug.
   - Steps to reproduce.
   - Expected vs actual behavior.
   - Your environment (OS, Rust version, relevant tools like `bwrap`/`mise`).
3. If you can, add a minimal reproduction or failing test.

## Proposing features

1. Open an issue describing:
   - The problem you want to solve.
   - Why it’s useful, especially for sandboxing agents/CLI tools.
   - Rough idea of your proposed solution.
2. Wait for feedback before investing heavily in an implementation, especially for bigger changes.

## Development workflow

### Prerequisites

- Rust (stable).
- `bwrap` installed and on `PATH`.
- Linux with user namespaces enabled.

### Setup

```bash
# Clone your fork
git clone git@github.com:<your-username>/jail.git
cd jail

# Build
cargo build

# Run tests
cargo test
```

### Branching

- Create a feature branch from `master`:

```bash
git checkout -b feature/my-change
```

- Keep your branch up to date with `master` and rebase if needed.

### Coding guidelines

- Follow Rust’s standard formatting:
  - Run `cargo fmt` before committing.
- Keep functions small and focused.
- Prefer explicitness over cleverness, especially around security / sandboxing logic.
- Add comments where behavior might be non-obvious (e.g. specific `bwrap` flags).

### Testing

Before opening a PR, run at least:

```bash
cargo fmt
cargo test
```

If you add or change behavior:

- Add or update unit tests.
- Consider adding integration-style tests if the behavior is user-facing.

## Pull requests

1. Make sure your changes are scoped and focused. Small PRs are easier to review.
2. Update documentation (README, examples, comments) if behavior or flags change.
3. Ensure CI is green (tests and formatting).
4. Open a PR against the `master` branch.

### PR title and description

- Use a clear, descriptive title (e.g. "Add --net flag to isolate network" instead of "Fix stuff").
- In the description, include:
  - Motivation / problem being solved.
  - High-level summary of the approach.
  - Any breaking changes or migrations.
  - How you tested it.

## Release process

Releases are created automatically from pushes to `master` based on the version in `Cargo.toml`.

When you introduce user-visible changes:

- Bump the version in `Cargo.toml` according to semantic versioning:
  - `MAJOR`: breaking changes.
  - `MINOR`: new features, backward compatible.
  - `PATCH`: bug fixes.
- Mention the change in the PR description so it can be included in release notes.

## Questions

If you’re unsure about anything, feel free to open an issue with the label `question` or start a draft PR and ask for early feedback.
