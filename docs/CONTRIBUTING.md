# Contributing to Nexforge Engine

## Getting Started

1. Ensure Rust toolchain 1.75+ is installed
2. Clone the repo: `git clone https://github.com/nexforge/nexforge-engine`
3. Build: `cargo build --workspace`
4. Test: `cargo test --workspace`

## Development Workflow

```bash
# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test --workspace

# Build release
cargo build --workspace --release
```

## Code Standards

- **Rust Edition**: 2021, **MSRV**: 1.75.0
- Zero `unwrap()` in production code — use `?` and `Result<T, E>`
- All errors use `thiserror` with `#[error("...")]` messages
- Every crate has `#![deny(clippy::all)]`
- All `pub` API has rustdoc with examples
- Follow **Conventional Commits**: `feat(scope):`, `fix(scope):`, `docs(scope):`, `chore(scope):`

## NexScript Guidelines

- All game logic is written in NexScript (`.nxs`), not Rust
- See `docs/NEXSCRIPT_SPEC.md` for language reference
- Game scripts live in `game/scripts/`

## Performance Budget

| Metric | Target |
|--------|--------|
| Frame time (gameplay) | < 16.6ms |
| ECS query (100k entity) | < 1ms |
| Physics step | < 4ms |
| NexScript VM cycle | < 0.1ms |

## Branch Strategy

- `main` — stable releases
- `develop` — integration branch
- `feat/*` — feature branches
- `fix/*` — bug fix branches

## Pull Request Process

1. Branch from `develop`
2. Implement your changes
3. Run `cargo fmt --check && cargo clippy -- -D warnings && cargo test --workspace`
4. Open PR targeting `develop`
5. Ensure CI passes
