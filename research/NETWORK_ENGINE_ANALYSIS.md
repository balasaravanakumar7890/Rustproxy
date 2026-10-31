# Network Engine Architecture & Protocol Analysis

This document provides in-depth technical analysis of the network protocols, TLS interception mechanics, and socket-level pipelines necessary to build a high-performance HTTP/HTTPS proxy in Rust.

---

## 1. HTTP/1.1 Wire Protocol Handling

### 1.1 Request & Response Framing
HTTP/1.1 message exchanges depend on strict line-ending rules (`\r\n` / CRLF) and message body delimitation:
- **Request Line**: `<Method> <Request-URI> <HTTP-Version>\r\n`
- **Status Line**: `<HTTP-Version> <Status-Code> <Reason-Phrase>\r\n`
- **Headers**: Array of `<Header-Name>: <Header-Value>\r\n`, terminated by empty line `\r\n`.
- **Body Framing**:
  1. `Content-Length: <n>`: Exactly `n` octets follow the header termination.
  2. `Transfer-Encoding: chunked`: Sequence of hexadecimal chunk size declarations, CRLF, chunk data, CRLF, culminating in a `0\r\n\r\n` trailer.
  3. Close-delimited body (Responses only): Server signals end of body by closing the TCP socket.

### 1.2 Defensive Wire Normalization
To prevent HTTP Request Smuggling (CL.TE or TE.CL desync vulnerabilities) between the proxy, client, and upstream server:
- If both `Content-Length` and `Transfer-Encoding: chunked` are present, `Transfer-Encoding` takes precedence according to RFC 9112, or the conflicting request is rejected with `400 Bad Request`.
- Strip sensitive hop-by-hop headers before relaying (`Connection`, `Keep-Alive`, `Proxy-Authenticate`, `Proxy-Authorization`, `TE`, `Trailers`, `Upgrade`).

---

## 2. HTTPS CONNECT Tunneling & TLS Interception

### 2.1 The CONNECT Tunnel Protocol
When a client intends to connect securely to `example.com:443` via an HTTP proxy, it issues:
```http
CONNECT example.com:443 HTTP/1.1
Host: example.com:443
User-Agent: Mozilla/5.0 ...
```
1. Proxy accepts connection.
2. Proxy parses destination host `example.com` and port `443`.
3. Proxy responds with:
```http
HTTP/1.1 200 Connection Established
Proxy-Agent: RustProxy/0.1.0

```
4. The TCP connection is now upgraded to a raw bidirectional byte stream.

### 2.2 TLS Interception Pipeline
```
Client Browser                         RustProxy                             Upstream Server
      |                                    |                                        |
      |--- 1. ClientHello (SNI: host) ---->|                                        |
      |                                    |--- 2. ClientHello (SNI: host) -------->|
      |                                    |<-- 3. ServerHello + Certificate -------|
      |                                    |    (Inspect upstream TLS params)       |
      |                                    |=== 4. Upstream TLS Established ========|
      |<-- 5. ServerHello + MITM Cert -----|                                        |
      |    (Signed by local Root CA)       |                                        |
      |=== 6. Downstream TLS Established ==|                                        |
      |                                    |                                        |
      |--- 7. Encrypted HTTP Request ----->|                                        |
      |                                    | [Decrypt -> Inspect -> Intercept/Log]  |
      |                                    |--- 8. Encrypted HTTP Request --------->|
      |                                    |<-- 9. Encrypted HTTP Response ---------|
      |                                    | [Decrypt -> Inspect -> Intercept/Log]  |
      |<-- 10. Encrypted HTTP Response ----|                                        |
```

### 2.3 Dynamic Certificate Authority (CA) Generation
1. **Root CA Generation**:
   - `RustProxy` generates an RSA-2048 or ECDSA P-256 self-signed Root Certificate Authority on first run:
     - Issuer/Subject: `CN=RustProxy Root CA, O=RustProxy Defensive Security, C=US`
     - Valid for 10 years.
     - Key usage: Certificate Sign, CRL Sign (`basicConstraints = critical, CA:TRUE`).
   - Exported as `rustproxy_ca.pem` and `rustproxy_ca.der` so operators can install it into their OS trust store or Firefox certificate store.
2. **On-the-Fly Leaf Certificate Synthesis**:
   - When a TLS `ClientHello` arrives, extract the Server Name Indication (SNI).
   - Check in-memory `LruCache<String, rustls::sign::CertifiedKey>`.
   - If not cached:
     - Generate a new leaf certificate with Subject Alternative Names (SAN) matching the SNI (e.g., `DNS:example.com`).
     - Sign using the Root CA's private key.
     - Cache in memory for subsequent connections to avoid cryptographic overhead.

---

## 3. WebSocket Interception (RFC 6455)

### 3.1 Upgrade Handshake
The client issues an HTTP/1.1 GET with:
```http
Connection: Upgrade
Upgrade: websocket
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
```
Upstream replies:
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

### 3.2 Frame Parsing & Inspection
Once upgraded, communication switches to binary frames:
- **Opcode**:
  - `0x1`: Text Frame (UTF-8)
  - `0x2`: Binary Frame
  - `0x8`: Connection Close
  - `0x9`: Ping
  - `0xA`: Pong
- **Masking**: Client-to-server frames are masked with a 4-byte XOR key. Server-to-client frames are unmasked.
- **Interception Hook**: RustProxy unmasks client frames, records the message in the WebSocket history, and applies filtering or tampering rules before re-masking and forwarding.

---

## 4. Connection Management & Concurrency Model

### 4.1 Tokio Task Architecture
- Every inbound client TCP socket is spawned onto its own Tokio async task (`tokio::spawn`).
- Upstream HTTP client utilizes `hyper::client::conn` with keep-alive connection pooling.
- Total memory per idle connection is ~10 KB in Rust, enabling thousands of concurrent connections with minimal RAM usage compared to JVM-based tools.
