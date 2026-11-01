# RustProxy Operator & Usage Guide

This guide provides step-by-step instructions for operating **RustProxy v1.0.0** to analyze, debug, and test authorized web applications.

---

## Safety & Defensive Use Notice

> [!IMPORTANT]
> RustProxy is engineered strictly for authorized web security assessments, local debugging, software development, and educational research on systems you own or have explicit written permission to test.
> Never inspect or alter traffic on third-party systems without authorization.

---

## 1. Launching RustProxy

Start the compiled binary from your terminal:

```bash
./target/release/rustproxy
```

At startup, RustProxy will:
1. Check for or generate a 10-year ECDSA Root Certificate Authority at `~/.rustproxy/rustproxy_ca.crt` (with private key at `~/.rustproxy/rustproxy_ca.key` locked to `0600` permissions).
2. Open or create the local SQLite flow database `rustproxy.db`.
3. Start the intercepting proxy listener on `http://127.0.0.1:8080`.
4. Start the local management REST API and GUI server on `http://127.0.0.1:8081`.

Open **`http://127.0.0.1:8081`** in any modern web browser or launch the Tauri desktop client.

---

## 2. Understanding the Dashboard

When the UI loads, the **Dashboard** displays live operational metrics:
- **Proxy Status**: Indicates whether the proxy listener is online and the active port.
- **Total Requests**: Total HTTP and HTTPS requests recorded in the active session.
- **HTTPS Encrypted**: Count of TLS connections intercepted.
- **Hosts Discovered**: Unique target domains identified in traffic.
- **Security Findings**: Real-time alerts detected by the passive scanner.
- **Recent Traffic Log**: Quick overview of the 5 most recent requests.

---

## 3. Configuring Browser Proxy

To route browser traffic through RustProxy, configure your browser's proxy settings to forward HTTP and HTTPS connections to loopback:

- **Proxy Type**: Manual Proxy Configuration
- **HTTP Proxy**: `127.0.0.1`
- **Port**: `8080`
- **HTTPS Proxy**: `127.0.0.1`
- **Port**: `8080`
- **Proxy DNS**: Checked (Recommended)

> [!TIP]
> **Recommended:** Use a dedicated testing browser profile or browser extension (such as *FoxyProxy*) to switch between Direct and RustProxy connections with one click.

---

## 4. Installing the Root CA Certificate

When inspecting encrypted HTTPS traffic, your browser must trust RustProxy's synthetic certificates signed by its local Root CA.

### Locating the Certificate
The certificate is generated at:
```text
~/.rustproxy/rustproxy_ca.crt
```

### Installation in Mozilla Firefox
1. Open Firefox Settings (`about:preferences`).
2. Search for **Certificates** and click **View Certificates...**
3. Select the **Authorities** tab and click **Import...**
4. Browse to and select `~/.rustproxy/rustproxy_ca.crt`.
5. In the trust prompt, check:
   - `[x] Trust this CA to identify websites`
6. Click **OK**.

### Installation in Google Chrome / Chromium
1. Open Chrome Settings (`chrome://settings/certificates`).
2. Navigate to **Privacy and security** > **Security** > **Manage certificates**.
3. Under the **Authorities** tab, click **Import**.
4. Select `~/.rustproxy/rustproxy_ca.crt` and confirm trust for website identification.

### Security Implications of Root CA Trust
> [!CAUTION]
> The Root CA allows RustProxy to decrypt and re-encrypt TLS connections on your machine.
> The private key (`~/.rustproxy/rustproxy_ca.key`) is generated with strict `0600` permissions and never leaves your local machine.
> Only install this certificate in test browsers or isolated testing virtual machines. Do not install it on production environments.

---

## 5. Capturing HTTP & HTTPS Traffic

With the proxy configured and CA certificate trusted:
1. Navigate to your authorized testing application in the proxied browser (e.g., `http://127.0.0.1:9099` or `https://authorized-test.local`).
2. All request lines, headers, cookies, query parameters, status codes, and bodies will stream into RustProxy in real time.

---

## 6. Using Traffic Interception

The **Proxy** tab enables live request and response inspection and tampering before traffic reaches upstream servers or returns to the browser.

### Intercept Toggle
- Click the **INTERCEPT IS ON / OFF** button in the top bar.
- When **ON**, matching traffic halts immediately and awaits operator decision.
- When **OFF**, traffic passes through automatically to the HTTP History.

### Inspecting and Modifying Outbound Requests
1. When a client sends a request with Intercept ON, the request pauses in the **Requests** sub-tab.
2. Select the request from the pending queue.
3. Modify any parameter:
   - Change HTTP Method (`GET` -> `POST`, etc.)
   - Alter the target URI
   - Add, edit, or delete HTTP Headers
   - Edit the JSON, XML, or raw Request Body
4. Click **Forward Request (Play icon)** to dispatch the modified request upstream.
5. Or click **Drop Request (Square icon)** to terminate the request with a `502 Bad Gateway` error.

### Inspecting and Modifying Inbound Responses
1. Switch to the **Responses** sub-tab.
2. When an upstream server responds, the response pauses before returning to the client browser.
3. Modify response status codes (e.g., `200 OK` -> `403 Forbidden` or `201 Created`), alter response headers, or inject test payloads into the response body.
4. Click **Forward Response** to deliver the modified payload to the client browser.

---

## 7. Inspecting HTTP History

Navigate to the **HTTP History** tab to examine traffic:
- **Search & Filter**: Type into the search input to filter by path, host, or status code in real time.
- **Detailed Inspection**: Click any row in the table to display the split detail pane:
  - **Request Tab**: Shows HTTP method, URI, headers, and formatted body.
  - **Response Tab**: Shows status code, response time, response headers, and body.
- **Send to Repeater**: Click **Send to Repeater** to clone the selected transaction for replay.
- **Send to Comparer**: Click **Send to Comparer** to send the payload to the diff engine.
- **Copy**: Quick-copy raw headers and bodies to system clipboard.

---

## 8. Manual Testing with Repeater

The **Repeater** allows you to craft, customize, and re-send individual HTTP requests:
1. Navigate to the **Repeater** tab.
2. Select an existing tab or click **New Tab (+)**.
3. Customize:
   - **Method**: Select `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `OPTIONS`, `HEAD`.
   - **URL**: Specify full target URI.
   - **Headers**: Add authentication tokens, cookies, or custom headers.
   - **Body**: Craft custom payloads (JSON, URL-encoded form data, etc.).
4. Click **Send (Ctrl+Enter)**.
5. The response pane immediately renders the server response, duration in ms, headers, and formatted body.
6. Click **Duplicate** to branch requests into variations.

---

## 9. Exploring the Sitemap Tree

Navigate to the **Target** tab:
- As you browse, RustProxy builds a visual tree structured by domain and hierarchical path components.
- View discovered endpoints, observed HTTP methods, status codes, and total request counts per path branch.

---

## 10. Viewing Passive Security Findings

Navigate to the **Scanner** tab:
- The passive engine non-intrusively analyzes observed flows in background.
- Select any finding to view:
  - **Severity**: High, Medium, Low, or Info badge.
  - **CWE Identification**: Common Weakness Enumeration reference.
  - **Affected URL**: Target endpoint.
  - **Description**: Technical explanation of the issue.
  - **Remediation**: Step-by-step guidance for developers to fix the weakness.
  - **Evidence Extract**: The exact header or cookie trigger observed in traffic.

---

## 11. Using Decoder & Encoder

Navigate to the **Decoder** tab:
1. Paste or type text into the **Input Payload** area.
2. Select target format:
   - **URL**: RFC 3986 percent-encoding / decoding.
   - **Base64**: RFC 4648 standard base64 encoding / decoding.
   - **Hex**: Hexadecimal byte conversion.
   - **HTML**: HTML character entity escaping / unescaping.
3. Click **Encode** or **Decode**.
4. Click **Use as Input** to chain transformations (e.g., URL-decode, then Base64-decode).

---

## 12. Using Comparer

Navigate to the **Comparer** tab:
1. Paste first text payload or response into **Pane A**.
2. Paste second payload into **Pane B**.
3. Click **Compare**.
4. The engine executes the Myers diff algorithm and visually highlights additions (green), deletions (red), and changes.

---

## 13. Stopping RustProxy Safely

To shut down RustProxy cleanly:
- Press **Ctrl+C** in the terminal where `rustproxy` is running.
- Tokio will close active listeners and SQLite will commit all WAL frames to disk cleanly.
