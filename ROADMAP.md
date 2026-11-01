# RustProxy Development Roadmap

This document outlines verified current capabilities and proposals for future development milestones.

> [!NOTE]
> Future version items represent architectural plans and active research tracks, not guaranteed delivery dates. Priorities are guided by community feedback and defensive security engineering requirements.

---

## Current Release: v1.0.0 (General Availability)

The current v1.0.0 release establishes a solid, high-performance foundation for local web application security testing and debugging:

- [x] **Asynchronous Proxy Engine**: Built with Tokio and Hyper 1.0; supports plaintext HTTP and HTTPS CONNECT tunneling.
- [x] **Dynamic TLS Certificate Synthesis**: 10-year ECDSA P-256 Root CA with on-demand leaf certificate generation matching requested SNI hostnames and IP SANs.
- [x] **Bidirectional Interception**: Outbound request and inbound response inspection, inline tampering, selective forwarding, and dropping.
- [x] **High-Performance HTTP History**: SQLite WAL embedded storage, indexing request metadata and preserving bit-for-bit wire bodies in BLOB storage.
- [x] **Tabbed Manual Request Editor (Repeater)**: Multi-tab request crafting, header customization, body editing, redirection tracking, and replay.
- [x] **Target / Sitemap Visualization**: Automatic discovery and aggregation tree showing endpoint counts, observed methods, and status codes.
- [x] **Passive Vulnerability Scanner**: Non-intrusive heuristics detecting missing security headers (CSP, HSTS, X-Content-Type-Options, clickjacking) and insecure cookie flags.
- [x] **Codec & Comparison Utilities**: Fast URL, Base64, Hex, and HTML codecs; Myers side-by-side textual diffing.
- [x] **Security Hardening**: Strict CORS validation on the local API and `0600` Unix file permissions on the Root CA private key.
- [x] **Desktop User Interface**: Dark-themed desktop GUI built with React 19, TypeScript, Tailwind CSS, and Tauri integration.

---

## Planned Milestone: v1.1 (Protocol Modernization & Real-Time Streams)

Focus on expanding protocol conformance to modern web technologies:

- [ ] **Native HTTP/2 Multiplexing**:
  - Full support for HTTP/2 ALPN negotiation (`h2`).
  - Concurrent stream multiplexing over a single TCP connection.
  - Frame-level inspection (HEADERS, DATA, SETTINGS, RST_STREAM).
- [ ] **WebSocket RFC 6455 Frame Dissection**:
  - Deep inspection of WebSocket upgrade handshakes.
  - Dedicated UI tab to view individual incoming and outgoing text, binary, ping, and pong frames.
  - Frame filtering and manual frame replay.
- [ ] **Streaming Body Enhancements**:
  - Non-buffering live streaming mode for Server-Sent Events (SSE) and chunked media streams.
- [ ] **Upstream Proxy Chaining**:
  - Support for upstream corporate HTTP and SOCKS5 proxy chains.

---

## Planned Milestone: v1.2 (Advanced Analysis & Workflow Optimization)

Focus on improving operator efficiency and deep application security diagnostics:

- [ ] **Enhanced Repeater Capabilities**:
  - Syntax highlighting for JSON, XML, GraphQL, and JavaScript.
  - Raw byte/hex editor mode for binary request construction.
  - Response search with regular expressions.
- [ ] **Advanced Traffic Filtering & Query DSL**:
  - Caido/Burp-style expressive query filter (e.g. `req.method == "POST" and res.status == 200 and res.body contains "token"`).
  - Saved filter presets.
- [ ] **Expanded Passive Security Heuristics**:
  - Detection of exposed secrets (JWTs, API keys, private tokens) in responses.
  - Insecure Cross-Origin Resource Sharing (CORS) wildcard reflection analysis.
  - Subresource Integrity (SRI) validation checks.
- [ ] **API Testing & OpenAPI Importer**:
  - Import Swagger / OpenAPI 3.0 specifications to pre-populate sitemaps and repeater endpoints.

---

## Future Research & Long-Term Roadmap

- [ ] **Extensibility & Plugin System**:
  - Safe, sandboxed WebAssembly (Wasm) or Rust dynamic plugin system for custom request transformation and passive rules.
- [ ] **HTTP/3 & QUIC Exploration**:
  - Research into UDP/QUIC tunneling and TLS 1.3 0-RTT inspection.
- [ ] **Cross-Platform Installers & Packaging**:
  - Verified packages for Debian/Ubuntu (`.deb`), Arch Linux (`PKGBUILD`), Fedora (`.rpm`), and macOS DMG.
- [ ] **Session Export & Archiving**:
  - HAR (HTTP Archive) export and import format support for sharing reproducible security issues with development teams.
