# RustProxy Feature Comparison Matrix

| Capability | Burp Suite Community / Pro | mitmproxy | Caido | RustProxy (Our Implementation) |
|---|---|---|---|---|
| **Core Language** | Java (JVM) | Python (Asyncio) | Rust + JS | **100% Native Rust** |
| **Memory Footprint (Idle)** | 400 MB - 1.2 GB | ~80 MB | ~30 MB | **~15 - 30 MB** |
| **Startup Time** | 5 - 15 seconds | 1 - 2 seconds | < 1 second | **< 300 milliseconds** |
| **HTTP/1.1 & HTTPS Proxy** | Yes | Yes | Yes | **Yes (Hyper + Rustls)** |
| **Dynamic Root CA & Leaf Certs**| Yes | Yes | Yes | **Yes (`rcgen` + SAN cache)** |
| **Real-time Request Interception**| Yes | Yes | Yes | **Yes (Async MPSC Channels)**|
| **HTTP History & Search** | Yes | Yes | Yes | **Yes (SQLite WAL + Indexes)**|
| **Manual Replay (Repeater)** | Yes | Limited | Yes | **Yes (Tabbed Replay Engine)**|
| **Target & Site Map Hierarchy** | Yes | No | Yes | **Yes (Radix Tree Sitemap)** |
| **Passive Security Audits** | Yes (Pro only) | No | Limited | **Yes (OWASP Best Practice Rules)**|
| **Decoder / Encoder Utilities** | Yes | No | Yes | **Yes (URL, B64, Hex, HTML, Gzip)**|
| **Differential Comparer** | Yes | No | No | **Yes (Myers Diff Engine)** |
| **Target Scope Enforcement** | Yes | Yes | Yes | **Yes (Regex & Prefix Scope Rules)**|
| **Desktop Interface** | Java Swing | Terminal (TUI) / Web | Tauri / Web | **Tauri + React Dark Theme UI**|
| **Sandboxed Extensions** | Java / Python / Montoya | Python scripts | JS / QuickJS | **WASM / Native Rust Plugins** |
