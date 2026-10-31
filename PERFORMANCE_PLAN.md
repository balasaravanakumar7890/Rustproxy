# RustProxy Performance & Benchmarking Plan

This document defines the performance goals, memory allocation constraints, and benchmarking methodology for the `RustProxy` engine.

---

## 1. Key Performance Indicators (KPIs)

| Metric | Target | Measurement Method |
|---|---|---|
| **Proxy Throughput (Plain HTTP)** | > 25,000 req/sec | `oha` / `wrk` benchmark tool |
| **Proxy Throughput (HTTPS Intercepted)** | > 5,000 req/sec | High-concurrency TLS benchmark |
| **Added Proxy Latency (p99)** | < 1.5 ms | Benchmarking local echo round-trip |
| **Idle Memory Consumption** | < 25 MB RSS | `ps`, `heaptrack`, or OS memory monitor |
| **Active Memory (100,000 flows in DB)** | < 80 MB RSS | Flow ingestion test with SQLite WAL |
| **Search Query Latency (100k flows)** | < 15 ms | Indexed SQLite query benchmark |
| **Certificate Synthesis & Cache Hit** | < 50 μs (cache hit), < 5 ms (cache miss) | Criterion microbenchmark |

---

## 2. Memory Optimization Architecture

1. **Zero-Copy Byte Buffers**: Use `bytes::Bytes` for body payloads to enable reference-counted zero-copy slicing across logger, interceptor, and proxy dispatcher tasks.
2. **LRU Certificate Cache**: Synthesized TLS certificates are cached using an LRU cache (`lru` crate) or `dashmap` indexed by SNI domain to prevent repeating RSA/ECDSA signing operations.
3. **Decoupled Database Persistence**: Traffic payloads are handed off to an asynchronous MPSC channel to avoid holding up the active network stream. SQLite runs in `PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;`.
4. **Streaming Response Payloads**: For files exceeding 10 MB, payloads are streamed directly without loading the full content into memory unless interception is explicitly activated.

---

## 3. Benchmark Scenarios

- **Scenario A (Raw Relay)**: 10,000 sequential HTTP/1.1 requests through the proxy to a local test HTTP server.
- **Scenario B (TLS Interception)**: 2,000 concurrent HTTPS connections with full TLS MITM certificate generation.
- **Scenario C (History Ingestion Under Load)**: Proxying continuous 1,000 req/sec traffic while performing complex regex searches on the history database.
