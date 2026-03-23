# AGENTS.md

## Cursor Cloud specific instructions

### Overview

This is **cursor-api** — a Rust nightly reverse-proxy / format-compatibility layer for the Cursor AI IDE API. It translates standard OpenAI/Anthropic API formats into Cursor's internal gRPC protocol.

Single binary, no databases, no Docker needed for development.

### Prerequisites (system-level, already installed in the VM)

- Rust **nightly** toolchain (`.rust-toolchain.toml` → `channel = "nightly"`)
- `protoc` (protobuf compiler) — required at build time
- Node.js + npm — required for build-time HTML/CSS/JS minification (`scripts/` directory)
- `pkg-config`, `libssl-dev` — required for OpenSSL linking

### Build & Run

| Action | Command |
|--------|---------|
| Dev build | `cargo build` |
| Release build | `cargo build --release` or `scripts/build.sh` |
| Lint | `cargo clippy` |
| Tests | `cargo test` |
| Run (dev) | `AUTH_TOKEN=<token> cargo run` or `AUTH_TOKEN=<token> ./target/debug/cursor-api` |

### Running the server

- The server listens on `0.0.0.0:3000` by default (configurable via `PORT` env var).
- `AUTH_TOKEN` env var is **required**; without it the binary exits immediately. Use any placeholder value for local dev (e.g. `AUTH_TOKEN=dev-token`).
- The `.env.example` file documents all available env vars. The app loads `.env` by default (if present).
- Health check: `GET /health` (no auth required).
- Model list: `GET /v1/models` (no auth required for listing).
- Chat completions require upstream Cursor session tokens provisioned via `/tokens/add`.

### Known issues

- `cargo test` may abort with a `ptr::copy_nonoverlapping` unsafe-precondition violation in `test_build_routes_deduplication`. This is a pre-existing issue in the test, not related to environment setup. Other tests pass.

### npm dependencies

Build-time minification scripts live in `scripts/`. Run `npm install` inside that directory to install them. This is needed before `cargo build` can embed minified assets.
