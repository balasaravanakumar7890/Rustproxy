# RustProxy Developer Guide

This document describes how to set up, build, test, and contribute to the **RustProxy** codebase.

---

## 1. Development Prerequisites

Ensure your development workstation has:
- **Rust Toolchain**: 1.80+ (stable channel) with `rustfmt` and `clippy`.
  ```bash
  rustup update stable
  rustup component add rustfmt clippy
  ```
- **Node.js**: v20.x or v22.x with `npm`.
- **System Libraries (Linux)**: `pkgconf`, `openssl`, `sqlite`, `webkit2gtk-4.1`.

---

## 2. Workspace Organization

The repository is organized as a Cargo workspace with 13 modular crates under `crates/`:

```text
Rustproxy/
├── Cargo.toml                  # Top-level workspace definition & shared dependencies
├── crates/
│   ├── rustproxy-core/         # Core data types (RequestModel, ResponseModel, Scope)
│   ├── rustproxy-tls/          # CA generation, leaf certificate synthesis, Rustls resolver
│   ├── rustproxy-storage/      # SQLite WAL engine & indexing
│   ├── rustproxy-http/         # Upstream Hyper HTTP client
│   ├── rustproxy-intercept/    # Interception queue & decision channels
│   ├── rustproxy-sitemap/      # URL tree aggregation
│   ├── rustproxy-repeater/     # Replay engine & tab management
│   ├── rustproxy-scanner/      # Passive heuristic security rules
│   ├── rustproxy-decoder/      # Multi-format codec (URL, Base64, Hex, HTML)
│   ├── rustproxy-comparer/     # Textual diffing engine
│   ├── rustproxy-proxy/        # Main proxy listener, CONNECT tunnel & event bus
│   ├── rustproxy-api/          # REST API & static file web server
│   └── rustproxy-cli/          # Application entrypoint binary
├── desktop/                    # React 19 + TypeScript + Tailwind CSS GUI
│   ├── src/                    # Frontend source components and state
│   ├── src-tauri/              # Tauri v2 native desktop wrapper
│   └── dist/                   # Compiled production frontend bundle
└── docs/                       # Technical specifications, screenshots, and guides
```

---

## 3. Building the Project

### Building Frontend Assets
```bash
cd desktop
npm ci
npm run build
cd ..
```
This bundles the React 19 single-page application into `desktop/dist/`.

### Checking the Workspace
```bash
source "$HOME/.cargo/env"
cargo check --workspace
```

### Compiling Debug Binary
```bash
cargo build --workspace
```

### Compiling Optimized Release Binary
```bash
cargo build --release --workspace
```

---

## 4. Running the Automated Test Suite

RustProxy maintains rigorous unit, integration, protocol, and concurrency tests.

### Running All Tests
```bash
cargo test --workspace
```

### Running Specific Test Suites
```bash
# Core unit tests
cargo test -p rustproxy-core

# TLS MITM synthesis tests
cargo test -p rustproxy-tls

# Protocol compatibility tests (Keep-Alive, Chunked, Large Payloads, Binary)
cargo test -p rustproxy-proxy --test protocol_test

# Concurrency & load tests (50 concurrent connections, interception queue)
cargo test -p rustproxy-proxy --test concurrency_test

# API & GUI static file serving integration test
cargo test -p rustproxy-api --test gui_integration_test
```

---

## 5. Code Style & Linting

Before opening a pull request, ensure the codebase complies with formatting and lint standards:

```bash
# Check formatting
cargo fmt --all -- --check

# Run Clippy lints
cargo clippy --workspace --all-targets -- -D warnings
```

For frontend code:
```bash
cd desktop
npm run build  # Validates TypeScript types (tsc) and Vite bundle
```
