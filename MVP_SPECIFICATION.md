# RustProxy MVP Specification

The Minimum Viable Product (MVP) of `RustProxy` delivers a functional, end-to-end intercepting proxy and traffic inspection suite.

---

## 1. MVP Scope & Capabilities

1. **Proxy Server Core (`rustproxy-proxy` & `rustproxy-http`)**:
   - HTTP/1.1 forward proxying on `127.0.0.1:8080`.
   - HTTPS `CONNECT` tunneling.
   - Dynamic TLS MITM interception using a generated local Root CA and SNI-based leaf certificates (`rustproxy-tls`).
2. **Traffic History & Persistence (`rustproxy-storage`)**:
   - Full recording of requests and responses into an embedded SQLite database (`rustproxy.db`).
   - Query API supporting pagination, method/status filtering, and search.
3. **Interception Engine (`rustproxy-intercept`)**:
   - Pause in-flight requests matching intercept criteria.
   - Forward, Drop, or inline Modify requests and responses.
4. **Manual Replay / Repeater (`rustproxy-repeater`)**:
   - Programmatic/UI tabbed request replay with raw HTTP editors.
   - Detailed timing and response metrics.
5. **Passive Security Auditing (`rustproxy-scanner`)**:
   - Automated checks for missing security headers (CSP, HSTS, X-Content-Type-Options, X-Frame-Options).
   - Cookie security flag auditing (`Secure`, `HttpOnly`, `SameSite`).
6. **Utility Suite (`rustproxy-decoder` & `rustproxy-comparer`)**:
   - Multi-format encoding/decoding (URL, Base64, Hex, HTML, Gzip).
   - Side-by-side Myers diff comparison for HTTP messages.
7. **Control Interface (`rustproxy-api` & CLI/Desktop)**:
   - Command-line runner and WebSocket/HTTP API enabling real-time event streaming and UI attachment.

---

## 2. Acceptance Criteria

- **AC-1**: Starting the proxy listens on `127.0.0.1:8080`.
- **AC-2**: Plain HTTP requests (e.g. `curl -x http://127.0.0.1:8080 http://httpbin.org/get` or local test server) pass through, get forwarded to upstream, and are logged into SQLite history.
- **AC-3**: HTTPS requests through the proxy with the generated Root CA trusted negotiate TLS, decrypt payloads, record in history, and return correct response to client.
- **AC-4**: When interception is enabled, matching requests are held in memory until explicit `forward` or `drop` command is issued.
- **AC-5**: Repeater can dispatch arbitrary HTTP requests to a target and capture the exact response with timing metrics.
- **AC-6**: Automated integration tests confirm proxy relay, TLS interception, and database query capabilities with 100% pass rate.
