# RustProxy Feature Specification & Capability Matrix

This document provides a comprehensive breakdown of all capabilities in **RustProxy v1.0.0**, organized by functional module.

---

## Capability Status Legend

- **Implemented and Tested**: Fully implemented in the Rust backend and Desktop UI, verified through unit, integration, and protocol tests.
- **Implemented (Limitations Documented)**: Implemented and functional for primary workflows; specific protocol or edge-case constraints apply.
- **Partially Supported**: Core infrastructure exists; deep inspection or full protocol conformance is in progress.
- **Planned**: Scheduled for upcoming releases (v1.1+); not present in v1.0.0.

---

## 1. Dashboard Module

- **Status**: Implemented and Tested.
- **Description**: Real-time operational telemetry and status visualization center.
- **User Perspective**:
  - Displays instant proxy listener status (`127.0.0.1:8080`), total requests captured, HTTPS requests handled, active unique hosts discovered, and security findings identified.
  - Features quick-action buttons to pause/resume interception, launch a new Repeater tab, and jump directly to traffic history or findings.
  - Summarizes the 5 most recent traffic flows and the latest passive vulnerability alerts.
- **Usage**:
  - Navigate to the **Dashboard** tab in the top navigation bar.
  - View real-time metric cards that update reactively as traffic flows through the proxy.
- **Limitations**:
  - Telemetry is scoped to the active session; historical multi-session trend analysis is planned for future releases.

---

## 2. Proxy & Interception Engine

- **Status**: Implemented and Tested.
- **Description**: Asynchronous HTTP/HTTPS intercepting proxy with bidirectional pause, inspect, tamper, forward, and drop capabilities.
- **User Perspective**:
  - **Master Toggle**: Intercept can be toggled on/off instantly via the UI toggle button or global controls.
  - **Outbound Request Interception**: Outbound client requests pause in an in-memory queue. The operator can edit the HTTP method, URI, headers, or body before forwarding upstream.
  - **Inbound Response Interception**: Inbound server responses pause for inspection. The operator can modify status codes, status text, headers, and response payloads before they reach the client browser.
  - **Drop Control**: Either the request or response can be dropped. Dropping an intercepted request or response returns a clear `502 Bad Gateway` error to the client.
  - **Send to Repeater**: Any intercepted request can be cloned directly into a new Repeater tab with one click.
- **Usage**:
  1. Configure your browser or HTTP client to proxy through `127.0.0.1:8080`.
  2. In the **Proxy** tab, ensure **Intercept is ON**.
  3. Issue a request from the browser; the request immediately appears in the **Requests** queue.
  4. Edit parameters in the raw or structured editor and click **Forward Request**.
- **Limitations**:
  - Streaming bodies (e.g., infinite Server-Sent Events) are buffered in memory when paused.

---

## 3. HTTP History Module

- **Status**: Implemented and Tested.
- **Description**: High-performance SQLite-backed persistent audit log of all HTTP and HTTPS flows.
- **User Perspective**:
  - Displays a tabular log containing Request #, Method, Status Code, Host, Path, MIME Type, Response Size, and Round-Trip Duration in milliseconds.
  - Supports live search across URLs, paths, and methods.
  - Selecting any row splits the screen into a dual-pane viewer displaying the complete Request and Response (headers, raw body, formatted payload).
  - Includes a **Clear History** button to reset session traffic.
- **Usage**:
  - Navigate to the **HTTP History** tab.
  - Click on any captured row to inspect the full HTTP transaction.
  - Use the search bar to filter by domain, keyword, or HTTP method.
- **Limitations**:
  - Full-text search over multi-gigabyte body contents is currently bounded to indexed metadata and path search queries.

---

## 4. Manual Request Editor (Repeater)

- **Status**: Implemented and Tested.
- **Description**: Tabbed HTTP request crafting, replay, and vulnerability validation workbench.
- **User Perspective**:
  - Supports multiple concurrent independent tabs.
  - Full control over HTTP method, target URL, request headers, and payload.
  - Displays response status code, round-trip execution duration, headers, formatted body, and raw bytes.
  - Supports tab duplication, renaming, and closing.
  - Synchronizes UI edits to the backend engine upon dispatch.
- **Usage**:
  - Navigate to the **Repeater** tab.
  - Click **New Tab** or clone an existing request from History/Intercept.
  - Modify parameters and click **Send (Ctrl+Enter)**.
- **Limitations**:
  - Replay engine operates over HTTP/1.1; native HTTP/2 frame crafting is planned for v1.1.

---

## 5. Target & Sitemap Module

- **Status**: Implemented and Tested.
- **Description**: Automated hierarchical discovery and endpoint aggregation tree.
- **User Perspective**:
  - Dynamically builds a hierarchical tree organized by Host -> Path components as traffic is observed.
  - Aggregates unique endpoints, observed HTTP methods (`GET`, `POST`), and status codes.
  - Displays endpoint counts and allows operators to visualize the application surface area.
- **Usage**:
  - Navigate to the **Target** tab to browse discovered domains and API routes.
- **Limitations**:
  - Passive discovery only; active recursive spidering/crawling is not included by design in the defensive core.

---

## 6. Passive Security Scanner

- **Status**: Implemented and Tested.
- **Description**: Non-intrusive rule-based security auditing engine analyzing live HTTP flows.
- **User Perspective**:
  - Evaluates captured responses without sending any additional or intrusive probe requests.
  - Generates detailed findings categorized by severity (`High`, `Medium`, `Low`, `Info`).
  - Includes CWE mapping, description, remediation guidance, and exact evidence extracts.
- **Active Detection Rules**:
  - Missing `Content-Security-Policy` (CSP) Header (CWE-1021)
  - Missing `X-Content-Type-Options: nosniff` (CWE-16)
  - Missing `X-Frame-Options` Clickjacking Protection (CWE-1021)
  - Missing `Strict-Transport-Security` (HSTS) on HTTPS connections (CWE-319)
  - Cookie Missing `HttpOnly` Flag (CWE-1004)
  - Cookie Missing `Secure` Flag on HTTPS (CWE-614)
  - Cookie Missing `SameSite` Attribute (CWE-1275)
  - Server Software Version Disclosure Header (CWE-200)
- **Usage**:
  - Navigate to the **Scanner** tab to view detected security alerts.
  - Select any finding to inspect evidence and remediation advice.
- **Limitations**:
  - Passive analysis only; does not perform active fuzzing, intrusive scanning, or exploit delivery.

---

## 7. Decoder & Encoder Utilities

- **Status**: Implemented and Tested.
- **Description**: Multi-format data conversion and payload manipulation utility.
- **User Perspective**:
  - Dual-pane layout: Input on the left, Output on the right.
  - Supports URL encoding/decoding (RFC 3986).
  - Supports Base64 encoding/decoding (RFC 4648).
  - Supports Hexadecimal encoding/decoding.
  - Supports HTML entity encoding/unescaping (`&lt;`, `&gt;`, `&amp;`, `&quot;`).
  - Quick-action buttons to copy output or cycle output back into input.
- **Usage**:
  - Navigate to the **Decoder** tab, enter or paste text, select target format, and click **Encode** or **Decode**.

---

## 8. Comparer Module

- **Status**: Implemented and Tested.
- **Description**: Side-by-side textual and cryptographic diff utility.
- **User Perspective**:
  - Compares two HTTP requests or responses to highlight modifications, token changes, or varying reflection lengths.
  - Computes additions, deletions, modifications, and exact line-by-line differences using the Myers diff algorithm.
- **Usage**:
  - Navigate to the **Comparer** tab, paste or send data into Pane A and Pane B, and click **Compare**.

---

## 9. Platform Settings & Trust Configuration

- **Status**: Implemented and Tested.
- **Description**: Management interface for listening interfaces, TLS certificate authority trust, and security boundaries.
- **User Perspective**:
  - Displays proxy bind address (`127.0.0.1:8080`) and API address (`127.0.0.1:8081`).
  - Displays generated Root CA certificate path (`~/.rustproxy/rustproxy_ca.crt`).
  - Provides a one-click button to copy certificate path to clipboard.
  - Features step-by-step instructions for importing and trusting the certificate in Firefox, Chrome, Chromium, and system trust stores.

---

## 10. Low-Level Core Engines

### Rust Asynchronous Networking Engine
- **Status**: Implemented and Tested.
- **Crate**: `rustproxy-proxy`, `rustproxy-http`
- **Features**: Built on Tokio async runtime and Hyper HTTP/1.1 engine. Supports HTTP CONNECT tunneling, persistent keep-alive connections, chunked transfer decoding, and high-concurrency connection handling (50+ simultaneous connections tested with <450ms throughput).

### Dynamic TLS MITM Engine
- **Status**: Implemented and Tested.
- **Crate**: `rustproxy-tls`
- **Features**: Generates a local 10-year ECDSA P-256 Root CA. Intercepts HTTPS tunnels and dynamically synthesizes valid X.509 v3 leaf certificates matching requested SNI hostnames and IP SANs (`127.0.0.1`, `::1`). Utilizes Rustls 0.23 with Ring crypto provider.

### SQLite WAL Persistence Engine
- **Status**: Implemented and Tested.
- **Crate**: `rustproxy-storage`
- **Features**: Embedded SQLite storage operating in Write-Ahead Logging (`WAL`) mode with `NORMAL` synchronicity. Thread-safe connection pooling, sub-millisecond indexed metadata lookups, and raw binary BLOB storage for full wire fidelity.

### Local REST & Static API Server
- **Status**: Implemented and Tested.
- **Crate**: `rustproxy-api`
- **Features**: Binds strictly to `127.0.0.1:8081`. Features strict CORS validation restricting access to `localhost`, `127.0.0.1`, and `tauri://localhost`. Serves embedded desktop GUI static assets and bidirectional JSON API endpoints.
