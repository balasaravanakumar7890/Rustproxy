use anyhow::Result;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use rustproxy_core::RequestModel;
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

async fn handle_api_request(
    req: Request<Incoming>,
    api: Arc<ApiService>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path();
    let query_str = uri.query().unwrap_or("");

    // Handle CORS Preflight
    if method == Method::OPTIONS {
        return Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
            .body(Full::new(Bytes::new()))
            .unwrap());
    }

    // Read request body for POST/PUT
    let body_bytes = match req.into_body().collect().await {
        Ok(c) => c.to_bytes(),
        Err(_) => Bytes::new(),
    };

    let res = match (method, path) {
        // Stats
        (Method::GET, "/api/stats") => match api.get_stats().await {
            Ok(stats) => json_response(StatusCode::OK, &stats),
            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
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
                Ok(flows) => json_response(StatusCode::OK, &flows),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
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
                    json_response(StatusCode::OK, &val)
                }
                Ok(None) => error_response(StatusCode::NOT_FOUND, "Flow not found"),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }

        // Clear Flows
        (Method::POST, "/api/flows/clear") => match api.clear_history() {
            Ok(_) => json_response(StatusCode::OK, &json!({ "success": true })),
            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        },

        // Intercept Status & Pending
        (Method::GET, "/api/intercept") => {
            let enabled = api.is_intercept_enabled();
            let pending = api.get_pending_intercepts().await;
            json_response(
                StatusCode::OK,
                &json!({
                    "enabled": enabled,
                    "pending": pending,
                }),
            )
        }

        // Toggle Intercept
        (Method::POST, "/api/intercept/toggle") => {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                let enabled = val.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                api.set_intercept_enabled(enabled);
                json_response(StatusCode::OK, &json!({ "enabled": enabled }))
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid JSON")
            }
        }

        // Forward Intercepted
        (Method::POST, "/api/intercept/forward") => {
            #[derive(serde::Deserialize)]
            struct ForwardPayload {
                id: String,
                modified: Option<RequestModel>,
            }
            if let Ok(payload) = serde_json::from_slice::<ForwardPayload>(&body_bytes) {
                let success = api.forward_intercepted(&payload.id, payload.modified).await;
                json_response(StatusCode::OK, &json!({ "success": success }))
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid forward payload")
            }
        }

        // Drop Intercepted
        (Method::POST, "/api/intercept/drop") => {
            #[derive(serde::Deserialize)]
            struct DropPayload {
                id: String,
            }
            if let Ok(payload) = serde_json::from_slice::<DropPayload>(&body_bytes) {
                let success = api.drop_intercepted(&payload.id).await;
                json_response(StatusCode::OK, &json!({ "success": success }))
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid drop payload")
            }
        }

        // Repeater: List Tabs
        (Method::GET, "/api/repeater/tabs") => {
            let tabs = api.list_repeater_tabs().await;
            json_response(StatusCode::OK, &tabs)
        }

        // Repeater: Create Tab
        (Method::POST, "/api/repeater/tabs") => {
            #[derive(serde::Deserialize)]
            struct CreateTabPayload {
                name: Option<String>,
                request: Option<RequestModel>,
            }
            let payload: CreateTabPayload = serde_json::from_slice(&body_bytes).unwrap_or(CreateTabPayload {
                name: None,
                request: None,
            });
            let tab_id = api.create_repeater_tab(payload.name, payload.request).await;
            json_response(StatusCode::CREATED, &json!({ "tab_id": tab_id }))
        }

        // Repeater: Duplicate Tab
        (Method::POST, p) if p.starts_with("/api/repeater/tabs/") && p.ends_with("/duplicate") => {
            let prefix = "/api/repeater/tabs/";
            let tab_id = &p[prefix.len()..p.len() - "/duplicate".len()];
            match api.duplicate_repeater_tab(tab_id).await {
                Some(new_id) => json_response(StatusCode::OK, &json!({ "tab_id": new_id })),
                None => error_response(StatusCode::NOT_FOUND, "Tab not found"),
            }
        }

        // Repeater: Close Tab
        (Method::DELETE, p) if p.starts_with("/api/repeater/tabs/") => {
            let tab_id = &p["/api/repeater/tabs/".len()..];
            api.close_repeater_tab(tab_id).await;
            json_response(StatusCode::OK, &json!({ "success": true }))
        }

        // Repeater: Send Request
        (Method::POST, p) if p.starts_with("/api/repeater/tabs/") && p.ends_with("/send") => {
            let prefix = "/api/repeater/tabs/";
            let tab_id = &p[prefix.len()..p.len() - "/send".len()];
            match api.send_repeater_tab(tab_id).await {
                Ok(res) => {
                    let body_str = res.body_as_string_lossy();
                    json_response(
                        StatusCode::OK,
                        &json!({
                            "response": res,
                            "body_str": body_str,
                        }),
                    )
                }
                Err(e) => error_response(StatusCode::BAD_GATEWAY, &format!("Request failed: {:?}", e)),
            }
        }

        // Sitemap
        (Method::GET, "/api/sitemap") => {
            let sitemap = api.get_sitemap().await;
            json_response(StatusCode::OK, &sitemap)
        }

        // Scanner Findings
        (Method::GET, "/api/findings") => match api.list_findings() {
            Ok(findings) => json_response(StatusCode::OK, &findings),
            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
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
                    Ok(result) => json_response(StatusCode::OK, &json!({ "result": result })),
                    Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
                }
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid codec payload")
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
                    Ok(result) => json_response(StatusCode::OK, &json!({ "result": result })),
                    Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
                }
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid codec payload")
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
                json_response(StatusCode::OK, &diff_res)
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid diff payload")
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

                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Full::new(Bytes::from(content)))
                    .unwrap()
            } else {
                error_response(StatusCode::NOT_FOUND, "Not Found")
            }
        }

        // Default 404
        _ => error_response(StatusCode::NOT_FOUND, "Endpoint not found"),
    };

    Ok(res)
}

fn json_response<T: serde::Serialize>(status: StatusCode, data: &T) -> Response<Full<Bytes>> {
    let json_bytes = serde_json::to_vec(data).unwrap_or_default();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(Full::new(Bytes::from(json_bytes)))
        .unwrap()
}

fn error_response(status: StatusCode, msg: &str) -> Response<Full<Bytes>> {
    let err_json = json!({ "error": msg });
    let bytes = serde_json::to_vec(&err_json).unwrap_or_default();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(Full::new(Bytes::from(bytes)))
        .unwrap()
}
