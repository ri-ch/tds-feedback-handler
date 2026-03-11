# CLAUDE.md

## Project overview

Lightweight Rust feedback-form handler deployed on Fly.io. Built with:

- **Axum** — web framework
- **deadpool-postgres** — PostgreSQL connection pool
- **Askama** — HTML templating
- CSRF protection + per-IP rate limiting

## Build & run

```bash
# Debug build
cargo build

# Production build (musl, stripped)
cargo build --release --profile size-opt2

# Run locally
RUST_LOG=debug TDS_FB_CONNECTION_STRING=<conn_str> cargo run
```

## Tests

```bash
cargo test      # 9 unit tests in src/main.rs
cargo clippy    # linting
cargo fmt       # formatting (rustfmt)
```

## Environment variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `TDS_FB_CONNECTION_STRING` | yes | — | PostgreSQL DSN (sslmode=require) |
| `PORT` | no | 8080 | HTTP listen port |
| `DB_POOL_SIZE` | no | 16 | Connection pool size |

## Key files

| Path | Purpose |
|---|---|
| `src/main.rs` | Routes, handlers, validation, tests |
| `src/appstate.rs` | DB pool + TLS initialisation |
| `src/errors.rs` | `AppError` → HTTP 500 wrapper |
| `src/tls.rs` | rustls config for PostgreSQL |
| `sql/feedbacktable.sql` | Schema (`CREATE TABLE IF NOT EXISTS`) |
| `templates/index.html` | Askama HTML template |
| `Dockerfile` | Production image (alpine 3.20, non-root) |
| `fly.toml` | Fly.io config (lhr, 1 shared CPU, 1 GB) |

## Code conventions

- Propagate errors with `?`; no `.unwrap()` / `.expect()` in handlers
- `tracing` macros for logging; level controlled by `RUST_LOG`
- Validation via `validator` derive macros on request structs
- MSRV: **1.75** (enforced in `Cargo.toml` and `rust-toolchain.toml`)
- Tokio runtime features: `rt-multi-thread`, `macros`, `net`, `signal` only

## CI/CD

- GitHub Actions (`fly-deploy.yml`): runs `cargo test` then `flyctl deploy` on push to `main`
- Fly health check: `GET /status` every 15 s
