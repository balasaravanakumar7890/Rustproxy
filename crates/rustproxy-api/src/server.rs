use anyhow::Result;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use rustproxy_core::{RequestModel, ResponseModel};
use rustproxy_storage::FlowFilter;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::ApiService;

pub async fn start_api_server(api: Arc<ApiService>, addr: SocketAddr) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!("RustProxy API Server listening on http://{}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let api = api.clone();
                let io = TokioIo::new(stream);
                tokio::spawn(async move {
                    let service = service_fn(move |req| {
                        let api = api.clone();
                        async move { handle_api_request(req, api).await }
                    });

                    if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                        tracing::debug!("API connection closed: {:?}", err);
                    }
                });
            }
            Err(e) => {
                error!("API accept error: {:?}", e);
            }
        }
    }
}

fn is_origin_allowed(origin: &str) -> bool {
    let lower = origin.to_lowercase();
    lower.starts_with("http://127.0.0.1")
        || lower.starts_with("https://127.0.0.1")
        || lower.starts_with("http://localhost")
        || lower.starts_with("https://localhost")
        || lower.starts_with("tauri://")
        || lower.starts_with("http://tauri.localhost")
        || lower.starts_with("https://tauri.localhost")
}

async fn handle_api_request(
    req: Request<Incoming>,
    api: Arc<ApiService>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path();
    let query_str = uri.query().unwrap_or("");

    // Determine Origin header and validate CORS
    let origin_header = req.headers().get("origin").and_then(|h| h.to_str().ok());
    let allowed_origin = match origin_header {
        Some(orig) if is_origin_allowed(orig) => Some(orig.to_string()),
        Some(_) => None, // Untrusted origin rejected
        None => None,    // Direct / non-browser access allowed without CORS headers
    };

    // Handle CORS Preflight
    if method == Method::OPTIONS {
        if origin_header.is_some() && allowed_origin.is_none() {
            // Reject unauthorized cross-origin requests
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Full::new(Bytes::from("CORS Origin Not Allowed")))
                .unwrap());
        }

        let mut builder = Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS",
            )
            .header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization",
            );

        if let Some(ref orig) = allowed_origin {
            builder = builder
                .header("Access-Control-Allow-Origin", orig)
                .header("Access-Control-Allow-Credentials", "true");
        }

        return Ok(builder.body(Full::new(Bytes::new())).unwrap());
    }

    // Reject untrusted cross-origin requests for state-changing/data API calls
    if origin_header.is_some() && allowed_origin.is_none() && path.starts_with("/api/") {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from("CORS Origin Not Allowed")))
            .unwrap());
    }

    // Read request body for POST/PUT
    let body_bytes = match req.into_body().collect().await {
        Ok(c) => c.to_bytes(),
        Err(_) => Bytes::new(),
    };

    let orig_ref = allowed_origin.as_deref();

    let res = match (method, path) {
        // Stats
        (Method::GET, "/api/stats") => match api.get_stats().await {
            Ok(stats) => json_response(StatusCode::OK, &stats, orig_ref),
            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string(), orig_ref),
        },

        // Flows List
        (Method::GET, "/api/flows") => {
            let mut filter = FlowFilter::all();
            for param in query_str.split('&') {
                if let Some((k, v)) = param.split_once('=') {
                    if k == "search" {
                        let decoded = urlencoding::decode(v).unwrap_or_default();
                        if !decoded.is_empty() {
                            filter.search_query = Some(decoded.to_string());
                        }
                    } else if k == "method" {
                        filter.method = Some(v.to_string());
                    } else if k == "in_scope" && v == "true" {
                        filter.in_scope_only = true;
                    } else if k == "limit" {
                        filter.limit = v.parse().unwrap_or(500);
                    }
                }
            }
            match api.list_flows(&filter) {
                Ok(flows) => json_response(StatusCode::OK, &flows, orig_ref),
                Err(e) => {
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string(), orig_ref)
                }
            }
        }

        // Single Flow Detail
        (Method::GET, p) if p.starts_with("/api/flows/") => {
            let id = &p["/api/flows/".len()..];
            match api.get_flow(id) {
                Ok(Some(flow)) => {
                    let req_body_str = flow.request.body_as_string_lossy();
                    let res_body_str = flow.response.as_ref().map(|r| r.body_as_string_lossy());
                    let val = json!({
                        "flow": flow,
                        "request_body_str": req_body_str,
                        "response_body_str": res_body_str,
                    });
                    json_response(StatusCode::OK, &val, orig_ref)
                }
                Ok(None) => error_response(StatusCode::NOT_FOUND, "Flow not found", orig_ref),
                Err(e) => {
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string(), orig_ref)
                }
            }
        }

        // Clear Flows
        (Method::POST, "/api/flows/clear") => match api.clear_history() {
            Ok(_) => json_response(StatusCode::OK, &json!({ "success": true }), orig_ref),
            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string(), orig_ref),
        },

        // Intercept Status & Pending
        (Method::GET, "/api/intercept") => {
            let enabled = api.is_intercept_enabled();
            let pending_requests = api.get_pending_intercepts().await;
            let pending_responses = api.get_pending_response_intercepts().await;
            json_response(
                StatusCode::OK,
                &json!({
                    "enabled": enabled,
                    "pending": pending_requests,
                    "pending_requests": pending_requests,
                    "pending_responses": pending_responses,
                }),
                orig_ref,
            )
        }

        // Toggle Intercept
        (Method::POST, "/api/intercept/toggle") => {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                let enabled = val.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                api.set_intercept_enabled(enabled);
                json_response(StatusCode::OK, &json!({ "enabled": enabled }), orig_ref)
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid JSON", orig_ref)
            }
        }

        // Forward Intercepted Request
        (Method::POST, "/api/intercept/forward") => {
            #[derive(serde::Deserialize)]
            struct ForwardPayload {
                id: String,
                modified: Option<RequestModel>,
            }
            if let Ok(payload) = serde_json::from_slice::<ForwardPayload>(&body_bytes) {
                let success = api.forward_intercepted(&payload.id, payload.modified).await;
                json_response(StatusCode::OK, &json!({ "success": success }), orig_ref)
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid forward payload", orig_ref)
            }
        }

        // Drop Intercepted Request
        (Method::POST, "/api/intercept/drop") => {
            #[derive(serde::Deserialize)]
            struct DropPayload {
                id: String,
            }
            if let Ok(payload) = serde_json::from_slice::<DropPayload>(&body_bytes) {
                let success = api.drop_intercepted(&payload.id).await;
                json_response(StatusCode::OK, &json!({ "success": success }), orig_ref)
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid drop payload", orig_ref)
            }
        }

        // Forward Intercepted Response
        (Method::POST, "/api/intercept/forward_response") => {
            #[derive(serde::Deserialize)]
            struct ForwardResponsePayload {
                id: String,
                modified: Option<ResponseModel>,
            }
            if let Ok(payload) = serde_json::from_slice::<ForwardResponsePayload>(&body_bytes) {
                let success = api
                    .forward_intercepted_response(&payload.id, payload.modified)
                    .await;
                json_response(StatusCode::OK, &json!({ "success": success }), orig_ref)
            } else {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "Invalid forward response payload",
                    orig_ref,
                )
            }
        }

        // Drop Intercepted Response
        (Method::POST, "/api/intercept/drop_response") => {
            #[derive(serde::Deserialize)]
            struct DropResponsePayload {
                id: String,
            }
            if let Ok(payload) = serde_json::from_slice::<DropResponsePayload>(&body_bytes) {
                let success = api.drop_intercepted_response(&payload.id).await;
                json_response(StatusCode::OK, &json!({ "success": success }), orig_ref)
            } else {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "Invalid drop response payload",
                    orig_ref,
                )
            }
        }

        // Repeater: List Tabs
        (Method::GET, "/api/repeater/tabs") => {
            let tabs = api.list_repeater_tabs().await;
            json_response(StatusCode::OK, &tabs, orig_ref)
        }

        // Repeater: Create Tab
        (Method::POST, "/api/repeater/tabs") => {
            #[derive(serde::Deserialize)]
            struct CreateTabPayload {
                name: Option<String>,
                request: Option<RequestModel>,
            }
            let payload: CreateTabPayload =
                serde_json::from_slice(&body_bytes).unwrap_or(CreateTabPayload {
                    name: None,
                    request: None,
                });
            let tab_id = api.create_repeater_tab(payload.name, payload.request).await;
            json_response(StatusCode::CREATED, &json!({ "tab_id": tab_id }), orig_ref)
        }

        // Repeater: Update Tab
        (Method::PUT, p) if p.starts_with("/api/repeater/tabs/") => {
            let tab_id = &p["/api/repeater/tabs/".len()..];
            #[derive(serde::Deserialize)]
            struct UpdateTabPayload {
                request: RequestModel,
            }
            if let Ok(payload) = serde_json::from_slice::<UpdateTabPayload>(&body_bytes) {
                let success = api.update_repeater_tab(tab_id, payload.request).await;
                json_response(StatusCode::OK, &json!({ "success": success }), orig_ref)
            } else {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "Invalid update tab payload",
                    orig_ref,
                )
            }
        }

        // Repeater: Duplicate Tab
        (Method::POST, p) if p.starts_with("/api/repeater/tabs/") && p.ends_with("/duplicate") => {
            let prefix = "/api/repeater/tabs/";
            let tab_id = &p[prefix.len()..p.len() - "/duplicate".len()];
            match api.duplicate_repeater_tab(tab_id).await {
                Some(new_id) => {
                    json_response(StatusCode::OK, &json!({ "tab_id": new_id }), orig_ref)
                }
                None => error_response(StatusCode::NOT_FOUND, "Tab not found", orig_ref),
            }
        }

        // Repeater: Close Tab
        (Method::DELETE, p) if p.starts_with("/api/repeater/tabs/") => {
            let tab_id = &p["/api/repeater/tabs/".len()..];
            api.close_repeater_tab(tab_id).await;
            json_response(StatusCode::OK, &json!({ "success": true }), orig_ref)
        }

        // Repeater: Send Request
        (Method::POST, p) if p.starts_with("/api/repeater/tabs/") && p.ends_with("/send") => {
            let prefix = "/api/repeater/tabs/";
            let tab_id = &p[prefix.len()..p.len() - "/send".len()];
            #[derive(serde::Deserialize)]
            struct SendTabPayload {
                request: Option<RequestModel>,
            }
            if let Ok(payload) = serde_json::from_slice::<SendTabPayload>(&body_bytes) {
                if let Some(req) = payload.request {
                    api.update_repeater_tab(tab_id, req).await;
                }
            }
            match api.send_repeater_tab(tab_id).await {
                Ok(res) => {
                    let body_str = res.body_as_string_lossy();
                    json_response(
                        StatusCode::OK,
                        &json!({
                            "response": res,
                            "body_str": body_str,
                        }),
                        orig_ref,
                    )
                }
                Err(e) => error_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("Request failed: {:?}", e),
                    orig_ref,
                ),
            }
        }

        // Sitemap
        (Method::GET, "/api/sitemap") => {
            let sitemap = api.get_sitemap().await;
            json_response(StatusCode::OK, &sitemap, orig_ref)
        }

        // Scanner Findings
        (Method::GET, "/api/findings") => match api.list_findings() {
            Ok(findings) => json_response(StatusCode::OK, &findings, orig_ref),
            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string(), orig_ref),
        },

        // Decoder: Encode
        (Method::POST, "/api/decoder/encode") => {
            #[derive(serde::Deserialize)]
            struct CodecPayload {
                text: String,
                format: String,
            }
            if let Ok(p) = serde_json::from_slice::<CodecPayload>(&body_bytes) {
                match api.encode(&p.text, &p.format) {
                    Ok(result) => {
                        json_response(StatusCode::OK, &json!({ "result": result }), orig_ref)
                    }
                    Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string(), orig_ref),
                }
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid codec payload", orig_ref)
            }
        }

        // Decoder: Decode
        (Method::POST, "/api/decoder/decode") => {
            #[derive(serde::Deserialize)]
            struct CodecPayload {
                text: String,
                format: String,
            }
            if let Ok(p) = serde_json::from_slice::<CodecPayload>(&body_bytes) {
                match api.decode(&p.text, &p.format) {
                    Ok(result) => {
                        json_response(StatusCode::OK, &json!({ "result": result }), orig_ref)
                    }
                    Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string(), orig_ref),
                }
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid codec payload", orig_ref)
            }
        }

        // Comparer: Diff
        (Method::POST, "/api/comparer/diff") => {
            #[derive(serde::Deserialize)]
            struct DiffPayload {
                old: String,
                new: String,
            }
            if let Ok(p) = serde_json::from_slice::<DiffPayload>(&body_bytes) {
                let diff_res = api.diff(&p.old, &p.new);
                json_response(StatusCode::OK, &diff_res, orig_ref)
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid diff payload", orig_ref)
            }
        }

        // Static Files for Desktop / Web GUI
        (Method::GET, p) => {
            let dist_dir = if std::path::Path::new("desktop/dist").exists() {
                std::path::PathBuf::from("desktop/dist")
            } else if std::path::Path::new("../../desktop/dist").exists() {
                std::path::PathBuf::from("../../desktop/dist")
            } else {
                std::path::PathBuf::from("desktop/dist")
            };

            let req_path = if p == "/" || p.is_empty() {
                "index.html"
            } else {
                p.trim_start_matches('/')
            };

            let file_path = dist_dir.join(req_path);
            let target_file = if file_path.is_file() {
                file_path
            } else {
                // Fallback for SPA routing
                dist_dir.join("index.html")
            };

            if let Ok(content) = std::fs::read(&target_file) {
                let mime = if target_file.extension().and_then(|e| e.to_str()) == Some("js") {
                    "application/javascript; charset=utf-8"
                } else if target_file.extension().and_then(|e| e.to_str()) == Some("css") {
                    "text/css; charset=utf-8"
                } else if target_file.extension().and_then(|e| e.to_str()) == Some("svg") {
                    "image/svg+xml"
                } else if target_file.extension().and_then(|e| e.to_str()) == Some("png") {
                    "image/png"
                } else {
                    "text/html; charset=utf-8"
                };

                let mut builder = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime);

                if let Some(orig) = orig_ref {
                    builder = builder.header("Access-Control-Allow-Origin", orig);
                }

                builder.body(Full::new(Bytes::from(content))).unwrap()
            } else {
                error_response(StatusCode::NOT_FOUND, "Not Found", orig_ref)
            }
        }

        // Default 404
        _ => error_response(StatusCode::NOT_FOUND, "Endpoint not found", orig_ref),
    };

    Ok(res)
}

fn json_response<T: serde::Serialize>(
    status: StatusCode,
    data: &T,
    origin: Option<&str>,
) -> Response<Full<Bytes>> {
    let json_bytes = serde_json::to_vec(data).unwrap_or_default();
    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", "application/json");

    if let Some(orig) = origin {
        builder = builder
            .header("Access-Control-Allow-Origin", orig)
            .header("Access-Control-Allow-Credentials", "true");
    }

    builder.body(Full::new(Bytes::from(json_bytes))).unwrap()
}

fn error_response(status: StatusCode, msg: &str, origin: Option<&str>) -> Response<Full<Bytes>> {
    let err_json = json!({ "error": msg });
    let bytes = serde_json::to_vec(&err_json).unwrap_or_default();
    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", "application/json");

    if let Some(orig) = origin {
        builder = builder
            .header("Access-Control-Allow-Origin", orig)
            .header("Access-Control-Allow-Credentials", "true");
    }

    builder.body(Full::new(Bytes::from(bytes))).unwrap()
}
