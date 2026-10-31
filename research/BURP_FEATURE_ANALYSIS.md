# Burp Suite Feature Analysis & Architectural Specification

This document presents a comprehensive technical analysis of the observable workflows, interface mechanics, and functional specifications of the core capabilities in web security testing proxies (such as PortSwigger Burp Suite, Caido, and mitmproxy), alongside the design strategy for an independent, defensive, memory-safe Rust platform (`RustProxy`).

---

## 1. Core Intercepting Proxy & Traffic Pipeline

### 1.1 Purpose
The intercepting proxy sits as a Man-in-the-Middle (MITM) between client user agents (browsers, mobile applications, HTTP clients) and upstream destination web servers. It allows security engineers, web developers, and defensive auditors to observe, filter, log, intercept, and modify HTTP/1.1, HTTP/2, and WebSocket traffic in real time.

### 1.2 User Interface & Workflow
- **Port Binding**: Configurable listeners (default `127.0.0.1:8080`).
- **Interception Toggle**: Global toggle button (`Intercept is on` / `Intercept is off`).
- **Control Actions**:
  - `Forward`: Dispatch the currently queued request/response to upstream/client.
  - `Drop`: Abort the connection, returning a dropped indicator or terminating the stream.
  - `Action menu`: Send to Repeater, Send to Comparer, Add to Scope.
- **Message Views**:
  - Raw editor (monospaced text with CRLF line breaks).
  - Pretty/Structured view (Key-Value header grid, formatted JSON/XML body).
  - Hex/Binary view for non-text payloads.

### 1.3 Observable Internal Workflow
```
Client Browser               Proxy Listener (RustProxy)               Upstream Server
      |                                  |                                   |
      |--- 1. Plain HTTP / CONNECT ----->|                                   |
      |    (or TLS Client Hello)         |                                   |
      |<-- 2. HTTP/1.1 200 OK (CONNECT)--|                                   |
      |--- 3. Client TLS Handshake ----->| [Inspect SNI, generate dynamic CA]|
      |<-- 4. MITM TLS Certificate ------|                                   |
      |=== 5. TLS Session Established ===|                                   |
      |                                  |--- 6. Upstream TLS Handshake ---->|
      |                                  |<-- 7. Upstream TLS Connected -----|
      |--- 8. Decrypted HTTP Request --->|                                   |
      |                                  |--[Evaluate Intercept Rules]--     |
      |                                  |  (If matching: halt in queue)     |
      |                                  |  (If forwarded: record in DB)     |
      |                                  |--- 9. Transmit HTTP Request ----->|
      |                                  |<-- 10. Upstream HTTP Response ----|
      |                                  |--[Evaluate Response Rules]--      |
      |<-- 11. Decrypted HTTP Response --|                                   |
```

### 1.4 Technical Requirements & Edge Cases
1. **HTTP CONNECT Handling**: Tunnels must identify whether the payload inside CONNECT is TLS. If non-TLS, proxy can operate as a transparent TCP tunnel.
2. **Dynamic Leaf Certificate Generation**: On-the-fly X.509 certificate generation with matching Subject Alternative Names (SANs) signed by a local Root Certificate Authority (CA).
3. **HTTP/1.1 Keep-Alive & Pipelining**: Connection pooling, handling persistent client and server sockets.
4. **HTTP/2 Multiplexing**: Supporting ALPN negotiation (`h2`, `http/1.1`). Downgrading or preserving HTTP/2 streams across proxy hops.
5. **WebSocket Upgrade**: Detecting `Upgrade: websocket` headers, completing the 101 Switching Protocols handshake, and shifting to duplex frame decoding.
6. **Chunked Transfer Encoding**: Normalizing or streaming chunked bodies without truncating premature connection drops.
7. **Content-Length & Transfer-Encoding Mismatches**: Defending against HTTP Request Smuggling during proxy relay.

### 1.5 Rust Implementation Strategy
- **Tokio & Hyper**: Asynchronous TCP listener, custom connection interceptor service.
- **Rustls & Rcgen**: Local CA generation (`rcgen` creates Root CA + dynamic leaf certs, cached by SNI in a concurrent DashMap / LruCache; `rustls` performs zero-cost TLS termination).
- **Tokio MPSC Channels**: Interception queue implemented as asynchronous priority channels where requests/responses await an operator decision (`Forward`, `Modify`, `Drop`).

---

## 2. HTTP History & Logging

### 2.1 Purpose
The HTTP History maintains a complete, persistent, searchable chronological ledger of every request and response passing through the proxy.

### 2.2 Functional Requirements
- **Sequential ID**: Monotonically increasing flow identifier (`#1`, `#2`, ...).
- **Metadata Fields**:
  - Request Method (GET, POST, PUT, DELETE, etc.).
  - Host (FQDN or IP, e.g., `api.internal.local`).
  - URL Path & Query (`/v1/auth/session?ref=web`).
  - HTTP Status Code (200, 302, 403, 500).
  - Response Length (Content length in bytes).
  - MIME / Content-Type (`application/json`, `text/html`).
  - Latency / Round-Trip Duration (milliseconds).
  - Timestamp (ISO 8601 UTC).
  - TLS Details (TLS 1.3 / TLS 1.2, Cipher suite, SNI).
  - Color highlight / Comment tagging.

### 2.3 Search and Filter Engine
- **Filter Bar Rules**:
  - Filter by In-Scope items only.
  - Filter by HTTP status code regex (`2xx`, `3xx`, `4xx`, `5xx`).
  - Filter by MIME type (HTML, Script, JSON, Images, Media).
  - Negative matching (hide static assets like `.png`, `.woff2`, `.css`).
  - Text search across Request Headers, Request Body, Response Headers, Response Body.

### 2.4 Rust Implementation Strategy
- **Persistence**: Embedded SQLite database in WAL (Write-Ahead Logging) mode.
- **Schema**:
  - `flows`: `id`, `session_id`, `timestamp`, `method`, `scheme`, `host`, `port`, `path`, `query`, `status_code`, `mime_type`, `request_size`, `response_size`, `duration_ms`, `in_scope`, `has_error`.
  - `flow_bodies`: `flow_id`, `request_raw`, `response_raw`, `request_body_blob`, `response_body_blob`.
- **Performance**: Normalized schema with blobs stored separately or compressed with zstd to ensure sub-millisecond query latency for 100,000+ flows.

---

## 3. Repeater (Manual HTTP Request Crafting & Replay)

### 3.1 Purpose
Repeater provides an isolated scratchpad for manual request editing, replaying, and response comparison. It allows engineers to inspect edge cases, assess server responses to malformed inputs, modify headers, and debug API behavior.

### 3.2 User Interface & Workflow
- **Tabbed Interface**: Dynamic tab list (`Tab 1`, `Tab 2`, ... `+`), ability to rename tabs, duplicate tabs, and group tabs.
- **Split Pane**: Request editor on the left (or top), Response viewer on the right (or bottom).
- **Controls**:
  - `Send` button (keyboard shortcut `Ctrl+Enter`).
  - `Cancel` button for in-flight requests.
  - History dropdown for the tab (`< Previous`, `Next >`).
  - Status indicator (e.g., `200 OK`, `34ms`, `1,428 bytes`).
  - Option to auto-follow redirects (301, 302, 307, 308) or manually step through.
  - Option to update `Content-Length` automatically upon typing.

### 3.3 Technical Requirements
- Support arbitrary raw HTTP bytes (including non-standard headers, missing headers, custom methods).
- Precise timing capture: Connection establishment time, TLS handshake time, Time to First Byte (TTFB), total transfer time.
- Syntax highlighting for JSON, XML, HTML, and Raw HTTP headers.

### 3.4 Rust Implementation Strategy
- Built on `hyper` and `rustls` using a raw HTTP engine that allows arbitrary header order, raw byte transmission, and exact wire replay without forced normalization.
- Request history stack per tab stored in SQLite/in-memory state.

---

## 4. Target & Site Map

### 4.1 Purpose
The Target module constructs a hierarchical, tree-structured site map of all discovered hosts, endpoints, and resources based on observed traffic, linking each node to its corresponding HTTP requests and responses.

### 4.2 Hierarchy Model
```
Target Hierarchy:
└── https://app.example.com
    ├── /
    ├── /api
    │   ├── /v1
    │   │   ├── /auth [POST 200, GET 401]
    │   │   └── /users
    │   │       ├── /{id} [GET 200, PUT 200]
    │   │       └── /profile [GET 200]
    └── /static
        ├── /css
        └── /js
```

### 4.3 Scope Definition
- **Scope Rules**:
  - Enabled / Disabled status.
  - Protocol (Any, HTTP, HTTPS).
  - Host regex (e.g., `.*\.example\.com$`, `10\.0\.0\..*`).
  - Port (Any, 80, 443, 8080).
  - Path prefix / regex (e.g., `^/api/.*`).
- **Scope Application**:
  - Proxy drop non-scope or pass-through without intercept.
  - Filter history and site map by "In-scope items only".
  - Prevent accidental inspection or traffic recording for third-party CDNs.

### 4.4 Rust Implementation Strategy
- **Radix Tree / Trie**: A concurrent prefix-tree structure in Rust indexing URLs by `(Scheme, Host, Port) -> Path segments -> NodeMetadata`.
- Each tree node maintains a list of flow IDs referencing the SQLite flow table.

---

## 5. Passive Security Configuration Analyzer

### 5.1 Purpose
The Passive Security Analyzer continuously examines proxy traffic in real time without sending additional requests. It inspects responses and headers against established security standards (OWASP ASVS, NIST, RFC specifications) to highlight insecure configurations.

### 5.2 Core Checks
1. **Security Headers**:
   - `Content-Security-Policy` (CSP): Missing, weak directives (e.g., `unsafe-inline`, `unsafe-eval`).
   - `Strict-Transport-Security` (HSTS): Missing, short max-age, missing `includeSubDomains`.
   - `X-Frame-Options` / `frame-ancestors`: Clickjacking defenses.
   - `X-Content-Type-Options`: Missing `nosniff`.
   - `Referrer-Policy`: Insecure leaking of sensitive URLs.
   - `Permissions-Policy`: Restricting camera, microphone, geolocation.
2. **Cookie Attributes**:
   - `Secure` flag missing over HTTPS.
   - `HttpOnly` flag missing on sensitive session identifiers.
   - `SameSite` attribute missing or set to `None` without `Secure`.
3. **Information Disclosure**:
   - Verbose `Server` banner or `X-Powered-By` headers exposing software versions.
   - Debug stack traces in error responses (`500 Internal Server Error`).
   - Private IP addresses or internal hostnames leaked in responses.
4. **CORS Configuration**:
   - `Access-Control-Allow-Origin: *` paired with credentials.
   - Arbitrary origin reflection.

### 5.3 Rust Implementation Strategy
- A rule-based analysis pipeline trait `PassiveRule` with signature:
  `fn analyze(&self, flow: &FlowRecord) -> Vec<SecurityFinding>;`
- Evaluated asynchronously on a worker thread after response completion to ensure zero proxy latency overhead.

---

## 6. Decoder & Comparer Utilities

### 6.1 Purpose
- **Decoder**: Fast multi-format conversion between raw text, encoded strings, and binary representations.
- **Comparer**: Differential analysis tool to highlight differences between two HTTP requests or responses (e.g., comparing responses to identify authorization bypasses or parameter reflections).

### 6.2 Encoding Capabilities
- URL Encoding / Decoding (RFC 3986, query component, full byte).
- Base64 & Base64URL (Standard, URL-safe, padding options).
- Hexadecimal (ASCII to Hex, Hex to ASCII, Hex dump view).
- HTML Entities (`&lt;`, `&gt;`, `&amp;`, decimal, hex entities).
- Compression: Gzip, Deflate, Brotli, Zstandard decompress/compress.
- Hashes: MD5, SHA-1, SHA-256, SHA-512, HMAC.
- JWT Decoder: Header / Payload JSON parsing and timestamp verification.

### 6.3 Comparer Capabilities
- Line-by-line diff.
- Word-by-word diff.
- Byte-level diff with hexadecimal side-by-side view.

### 6.4 Rust Implementation Strategy
- Pure Rust crates: `base64`, `hex`, `urlencoding`, `html-escape`, `flate2`, `similar` (high-performance Myers diff implementation in Rust).

---

## 7. Extension System

### 7.1 Purpose
Allows custom plugins to extend the proxy pipeline, register custom passive rules, add UI panels, or modify traffic flows programmatically.

### 7.2 Safe Sandboxing Architecture
- **WebAssembly (WASM) Engine**: Using `wasmtime` for sandboxed, high-performance plugin execution with capability-based security.
- **Hook Lifecycle**:
  - `on_request(&mut Request)`
  - `on_response(&mut Response)`
  - `on_passive_scan(&Flow) -> Vec<Finding>`
- **Safety**: Sandboxed plugins cannot access filesystem or network sockets unless explicitly granted by the operator.
