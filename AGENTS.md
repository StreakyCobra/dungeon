# AGENTS.md

Guidance for coding agents working in this repository.

## Scope

- Applies to the full repo rooted at `/home/dungeon/dungeon`.
- Primary language: Rust (edition 2024).
- App type: CLI (`dungeon`) for Podman/Docker wrapper workflows.

## Repository Layout (high value paths)

- `src/main.rs`: process entrypoint, prints errors and exits with app error code.
- `src/app.rs`: top-level command dispatch and run flow.
- `src/cli.rs`: clap command construction + argument parsing + validation.
- `src/config/`: config loading, parsing, merge, and group resolution.
- `src/container/engine.rs`: command generation and subprocess execution.
- `src/container/persist.rs`: persisted container lifecycle logic.
- `src/tests/`: unit/integration-style tests with deterministic command assertions.
- `src/config/defaults.toml`: embedded default configuration.
- `README.md`: user-facing behavior and config examples.

## Build / Lint / Test Commands

Run from repo root.

### Build

- Debug build: `cargo build`

### Format / Lint

- Format code: `cargo fmt`
- Check formatting only: `cargo fmt --check`
- Lint (if available in environment): `cargo clippy --all-targets --all-features -- -D warnings`

### Test

- Full suite: `cargo test`
- Show output while running: `cargo test -- --nocapture`
- Run one test by exact name:
  - `cargo test tests::validation::errors_on_unknown_config_keys -- --exact`
- Run tests matching a substring:
  - `cargo test group_overrides`
- Run one test module/file target:
  - `cargo test tests::engine`

### Quick Runtime Smoke Checks

- Top-level help: `cargo run -- --help`
- Run-subcommand help: `cargo run -- run --help`
- Version: `cargo run -- --version`

## Coding Style Conventions

Follow existing patterns in `src/` and `src/tests/`.

### Formatting and Structure

- Use rustfmt defaults; do not hand-format around it.
- Keep modules small and focused (`app`, `cli`, `config`, `container`, `error`).
- Prefer small helper functions over deeply nested control flow.
- Keep public API minimal; default to private helpers.

### Imports

- Group imports by origin:
  1. `std`
  2. external crates
  3. `crate::...`
- Use grouped imports when readable (e.g., `use crate::{cli, config, ...};`).
- Avoid wildcard imports.

### Naming

- Types/enums/traits: `UpperCamelCase`.
- Functions/variables/modules: `snake_case`.
- Constants: `SCREAMING_SNAKE_CASE`.
- Keep CLI flag/key constants centralized and reused.
- Use descriptive names (`persist_mode`, `group_defs`, `run_command`).

### Types and Data Modeling

- Use explicit structs for domain state (`Settings`, `Config`, `ResolvedConfig`).
- Prefer enums for constrained choices (`Engine`, action enums, persist mode).
- Use `Option<T>` for optional config values.
- Favor `Vec<String>` for ordered repeatable CLI/config entries.
- Use `BTreeMap` when stable ordering helps deterministic behavior/tests.

### Error Handling

- Use `Result<T, AppError>` for fallible operations.
- Prefer `?` to propagate errors.
- Create user-facing errors with `AppError::message(...)`.
- Keep error messages actionable and specific; include context path/key/name.
- Use `ERROR:` prefix in user-facing validation errors when consistent with existing code.
- Return typed subprocess failures as `AppError::Subprocess(code, msg)`.

### Unsafe Code

- Unsafe is currently used in tests for env var mutation and in UID/GID calls.
- Keep unsafe blocks minimal and tightly scoped.
- Do not introduce new unsafe code unless no safe alternative is practical.

### CLI and UX Behavior

- Validate conflicting flags early in parse/validation functions.
- Keep help text precise and synchronized with actual behavior.
- Preserve established precedence: CLI > env > groups > file > defaults.
- Avoid silently ignoring invalid configuration keys.

### Testing Conventions

- Add tests under `src/tests/` via `src/tests/mod.rs` module listing.
- Prefer deterministic string-based assertions of generated command lines.
- Use `tests::support::{TestInput, assert_command, run_input}` utilities.
- Use `std::panic::catch_unwind` when asserting error paths in helper-driven tests.
- Normalize host-specific values in expected strings (`<CWD>`, `<HOME>`, `<UID>:<GID>`).

## Change Checklist for Agents

Before finishing a change:

1. Run `cargo fmt`.
2. Run targeted tests for touched logic.
3. Run `cargo test` for full validation.
4. If CLI/config behavior changed, update `README.md` examples and docs.
5. If defaults or env keys changed, update tests and support constants accordingly.

## Things to Avoid

- Do not add backward compatibility unless requested (project is alpha).
- Do not weaken validation to accept ambiguous/unknown config keys.
- Do not change command-building order unless tests and docs are updated together.
- Do not introduce unrelated refactors in feature/fix PRs.

## Notes for Future Agents

- If repository-level agent rules are introduced later (`.cursor/rules`, `.cursorrules`,
  or `.github/copilot-instructions.md`), update this file to summarize them and follow them.
- Keep this document concise, operational, and synced with actual commands in CI/developer flow.
