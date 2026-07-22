## Overview

`dungeon` is a Rust CLI for launching sandboxed development containers on top of Podman. It supports layered configuration, reusable group presets, image building, cache management, and an in-container bootstrap that applies network policy before dropping privileges.

## Main Parts

- CLI parsing and command routing: `src/cli/`
- App and config orchestration: `src/app.rs`, `src/config/`
- Podman command construction and execution: `src/container/`
- Container image and bootstrap scripts: `images/`
- Tests: `src/tests/`
- Product docs: `README.md`

## Potential Improvements

1. Harden runtime security defaults
   - Refs: `src/container/engine.rs:79-141`, `README.md:191-225`
   - The default sandbox configuration appears broad for a security-sensitive tool: root user, elevated capabilities, and `seccomp=unconfined`.

2. Add stricter mount and path validation
   - Refs: `src/container/engine.rs:156-194`, `src/tests/paths.rs:34-45`, `README.md:181`
   - Raw host mount specs and permissive explicit paths can cause confusing failures or accidental exposure of host directories.

3. Add real Podman integration tests
   - Refs: `src/tests/basic_run.rs`, `src/tests/network.rs`, `src/tests/image_cache.rs`
   - Current coverage is mostly command-shape and unit-level verification, not real container startup or bootstrap policy behavior.

4. Add CI for formatting, linting, tests, and smoke tests
   - Refs: repository root, no `.github/` workflows present
   - Automated validation would catch regressions in CLI, config, and runtime behavior earlier.

5. Refactor config and CLI merging into a more declarative typed model
   - Refs: `src/config/parse.rs`, `src/config/merge.rs`, `src/cli/parse.rs`, `src/config/types.rs`
   - The current field-by-field parsing and merge logic likely increases maintenance cost and drift risk.

6. Improve subprocess diagnostics and preflight checks
   - Refs: `src/container/mod.rs`, `src/error.rs`, `src/container/engine.rs`
   - Failures appear relatively generic; showing the exact failing command and missing prerequisites would improve operability.

7. Tighten `dungeon-install` guarantees and document the threat model
   - Refs: `images/dungeon-install`, `README.md:227-233`
   - The helper is useful, but its safety boundary should be more explicit and resilient over time.

8. Restructure documentation
   - Refs: `README.md`
   - The README currently combines product overview, configuration reference, and security notes; splitting it would improve usability and auditability.

9. Reduce test reliance on global environment and working-directory mutation
   - Refs: `src/tests/support.rs`
   - Process-wide mutation plus locking is workable but brittle and makes cleaner parallel test execution harder.

10. Remove unused complexity and dependencies
   - Refs: `Cargo.toml`
   - `serde` appears declared but not obviously used; there may be opportunities to simplify the dependency set and related code.
