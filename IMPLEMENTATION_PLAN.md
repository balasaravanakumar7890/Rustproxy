# RustProxy Implementation Plan & Roadmap

This document outlines the step-by-step phased execution plan to build `RustProxy` incrementally, verifying every milestone with automated tests before advancing.

---

## Phase Breakdown & Milestones

### Milestone 1: Research, Specifications & Architecture (Current)
- [x] Feature analysis (`research/BURP_FEATURE_ANALYSIS.md`)
- [x] UI analysis (`research/UI_ANALYSIS.md`)
- [x] Network engine analysis (`research/NETWORK_ENGINE_ANALYSIS.md`)
- [x] Architecture specification (`ARCHITECTURE.md`)
- [x] Implementation roadmap (`IMPLEMENTATION_PLAN.md`)
- [x] Feature comparison matrix (`FEATURE_MATRIX.md`)
- [x] Security model & boundaries (`SECURITY_MODEL.md`)
- [x] Performance plan (`PERFORMANCE_PLAN.md`)

### Milestone 2: Rust Workspace Setup & Core Domain Layer
- [ ] Initialize Cargo workspace with root `Cargo.toml`.
- [ ] Implement `rustproxy-core`:
  - `FlowId`, `Timestamp`, `ProtocolVersion`
  - `HttpRequest` and `HttpResponse` structs (headers, method, URI, status, raw bytes)
  - `FlowRecord` representing an end-to-end transaction
  - Unit tests for serialization, header manipulation, and model conversions.

### Milestone 3: TLS Engine & Dynamic Certificate Authority
- [ ] Implement `rustproxy-tls`:
  - Generate self-signed Root CA (`rcgen`) and write PEM/DER outputs.
  - In-memory dynamic leaf certificate synthesis with Subject Alternative Names (SANs).
  - In-memory certificate cache (`dashmap` or `lru`).
  - `rustls` ServerConfig and ClientConfig builders.
  - Unit tests: Generate cert, verify signature against root CA, validate SAN resolution.

### Milestone 4: Storage Engine (SQLite WAL)
- [ ] Implement `rustproxy-storage`:
  - SQLite database initialization with schema migrations (`rusqlite`).
  - Table definitions: `flows`, `flow_bodies`, `scope_rules`, `findings`.
  - Batching background actor (`tokio::sync::mpsc`) for zero-latency logging.
  - Queries: pagination, full-text search, status/method filters.
  - Unit tests: Write 1,000 flows, query with filters, verify data integrity.

### Milestone 5: HTTP/HTTPS Intercepting Proxy Core
- [ ] Implement `rustproxy-http` and `rustproxy-proxy`:
  - Tokio TCP listener on `127.0.0.1:8080`.
  - HTTP/1.1 plain proxying.
  - HTTPS `CONNECT` tunnel handling and TLS handshake interception.
  - Upstream request dispatch and response streaming.
  - End-to-end integration test with local HTTP & HTTPS echo servers.

### Milestone 6: Interception Queue & Scope Evaluator
- [ ] Implement `rustproxy-intercept`:
  - Queue manager with `Forward`, `Drop`, and `Modify` actions.
  - Rule matcher (Method, Host, Path regex, Header values).
- [ ] Implement `rustproxy-sitemap`:
  - Scope evaluator (in-scope regex, port, scheme).
  - Radix tree site map hierarchy generator.
  - Tests: Intercept request, modify header, forward, verify upstream received modified header.

### Milestone 7: Manual Replay Engine (Repeater)
- [ ] Implement `rustproxy-repeater`:
  - Tab manager with isolated request/response history.
  - Raw HTTP wire sender using `hyper` / `rustls`.
  - Precise round-trip timing (DNS, TLS, TTFB, Total).
  - Tests: Send request, follow redirects, verify response recording.

### Milestone 8: Passive Security Scanner
- [ ] Implement `rustproxy-scanner`:
  - Rule registry for passive checks.
  - Rules: CSP, HSTS, X-Content-Type-Options, X-Frame-Options, cookie security flags (Secure, HttpOnly, SameSite).
  - Sensitive information exposure checks (stack traces, server headers).
  - Tests: Scan vulnerable responses, verify findings generated with accurate severity.

### Milestone 9: Decoder & Comparer Utilities
- [ ] Implement `rustproxy-decoder`:
  - URL, Base64, Hex, HTML entity, Gzip codecs.
- [ ] Implement `rustproxy-comparer`:
  - Line-by-line and byte-level diff using `similar`.
  - Tests: Diff two HTTP responses, verify insertions and deletions.

### Milestone 10: API Server & Desktop Frontend Integration
- [ ] Implement `rustproxy-api`:
  - WebSocket and JSON-RPC endpoints for real-time flow streaming and queue actions.
- [ ] Setup Desktop interface (Tauri + React/TS or fast web dashboard):
  - Dark theme UI with History table, Intercept panel, Repeater tabs, and Scanner findings.
  - End-to-end operational verification.
