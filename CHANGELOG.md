# Changelog

All notable changes to **RustProxy** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-11-01

### Initial Stable Release

#### Added
- **Core Asynchronous Proxy**:
  - Implemented high-performance HTTP/1.1 and HTTPS CONNECT tunneling proxy using Tokio and Hyper 1.0.
  - Added support for persistent HTTP/1.1 keep-alive connections and chunked transfer decoding.
  - Verified 50+ concurrent client connections and 5 MB large body transfers with bit-level binary transparency.
- **Dynamic TLS Interception**:
  - On-demand X.509 v3 leaf certificate synthesis signed by a local ECDSA P-256 Root CA.
  - Automatic Subject Alternative Name (SAN) mapping for requested SNI hostnames and loopback IP addresses (`127.0.0.1`, `::1`).
  - Integrated Rustls 0.23 with Ring crypto provider.
- **Traffic Interception & Modification**:
  - Bidirectional request and response interception queues.
  - Inline editing of HTTP methods, URIs, status codes, status text, headers, and request/response payloads before forwarding.
  - Drop control with structured `502 Bad Gateway` error responses.
- **Persistent HTTP History**:
  - Embedded SQLite storage operating in Write-Ahead Logging (`WAL`) mode with `NORMAL` synchronicity.
  - Indexed sub-millisecond retrieval of traffic metadata (`flows`).
  - Bit-for-bit raw wire payload storage in binary `BLOB` fields (`flow_bodies`).
- **Repeater (Manual Request Replay Engine)**:
  - Multi-tab HTTP request crafting interface with method selectors, URI editor, header tables, and payload editor.
  - Immediate execution with response timing in milliseconds, status code badges, header inspection, and formatted response viewing.
  - Tab duplication, renaming, and history tracking.
- **Target / Sitemap**:
  - Hierarchical tree visualization dynamically grouping endpoints by host and path segments.
  - Aggregation of unique endpoints, observed HTTP methods, and status codes.
- **Passive Security Heuristics**:
  - Non-intrusive rule engine auditing captured flows.
  - Detection rules for missing `Content-Security-Policy` (CSP), `X-Content-Type-Options`, `X-Frame-Options` (Clickjacking), and `Strict-Transport-Security` (HSTS).
  - Detection of missing `HttpOnly`, `Secure`, and `SameSite` flags on cookies.
  - Server software version disclosure detection (CWE-200).
- **Utility Modules**:
  - Multi-format Decoder/Encoder supporting URL, Base64, Hexadecimal, and HTML entity conversion.
  - Comparer implementing Myers textual diff algorithm for side-by-side payload inspection.
- **Security & Workstation Protection**:
  - Strict Cross-Origin Resource Sharing (CORS) origin validation protecting the local management API from unauthorized cross-origin browser access.
  - Strict Unix permissions enforced on Root CA directory (`0700`) and private key file (`0600`).
  - Loopback-only binding on `127.0.0.1:8080` (proxy) and `127.0.0.1:8081` (API/GUI).
- **Desktop Graphical Interface**:
  - React 19, TypeScript, and Tailwind CSS dark-themed user interface.
  - Tauri v2 native desktop integration and embedded web GUI serving.
- **Documentation & Verification**:
  - Comprehensive quality audit (`QUALITY_AUDIT.md`) verifying all 21 automated test suites in the workspace.
  - Technical architecture specification, installation guide, and step-by-step usage documentation.
