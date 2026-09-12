mod tools_call;
mod tools_list;

use actix_web::{web, HttpResponse, Result, HttpRequest};
use crate::AppState; // adjust to your actual path
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json;

pub fn configure(cfg: &mut web::ServiceConfig) {
cfg.route("/mcp", web::post().to(handle_rpc))
   .route("/mcp", web::get().to(|| async {
       HttpResponse::MethodNotAllowed().finish()
   }))
   .route("/icon.svg", web::get().to(|| async {
            HttpResponse::Ok()
                .content_type("image/svg")
                .body(include_bytes!("../../icon.svg").to_vec())
    }));
}

#[derive(Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    #[serde(default)]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

pub async fn handle_rpc(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<RpcRequest>,
) -> Result<HttpResponse> {

    if body.method.as_str() == "notifications/initialized" {
        return Ok(HttpResponse::Accepted().finish());
    }

    let response = match body.method.as_str() {
        "tools/call" => {
            let params: ToolCallParams = match serde_json::from_value(body.params.clone()) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(HttpResponse::Ok().json(RpcResponse {
                        jsonrpc: "2.0",
                        id: body.id.clone(),
                        result: None,
                        error: Some(RpcError {
                            code: -32602,
                            message: format!("Invalid params: {e}"),
                        }),
                    }));
                }
            };

            let input = params.arguments.get("input").and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            match tools_call::tools_call(&state.client, &state.config, &params.name, &input).await {
                Ok(result) => RpcResponse {
                    jsonrpc: "2.0",
                    id: body.id.clone(),
                    result: Some(serde_json::json!({
                        "content": [{ "type": "text", "text": result }]
                    })),
                    error: None,
                },
                Err(e) => RpcResponse {
                    jsonrpc: "2.0",
                    id: body.id.clone(),
                    // Tool execution failures go back as isError, not a JSON-RPC error
                    result: Some(serde_json::json!({
                        "content": [{ "type": "text", "text": e.to_string() }],
                        "isError": true
                    })),
                    error: None,
                },
            }
        }

        "tools/list" => RpcResponse {
            jsonrpc: "2.0",
            id: body.id.clone(),
            result: Some(serde_json::json!({
                "tools": tools_list::tools_list()
            })),
            error: None,
        },

        "initialize" => {

        let conn = req.connection_info();
        let scheme = conn.scheme();
        let host = conn.host();

        RpcResponse {
            jsonrpc: "2.0",
            id: body.id.clone(),
            result: Some(serde_json::json!({
                "protocolVersion": "2025-06-18",
                // "protocolVersion": "2026-07-28",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "rust-mcp",
                    "title": "rust-mcp",
                    "version": "0.1.0",
                    "icons": [
                        {
                            "src": format!("{}://{}/icon.svg", scheme, host),
                            "mimeType": "image/svg+xml",
                            "sizes": ["48x48"]
                        }
                    ]
                }
            })),
            error: None,
        }
        },

        other => RpcResponse {
            jsonrpc: "2.0",
            id: body.id.clone(),
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: format!("Method not found: {other}"),
            }),
        },
    };

    Ok(HttpResponse::Ok().json(response))
}
