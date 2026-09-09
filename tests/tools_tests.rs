use actix_web::{http::StatusCode, test};
use rust_mcp::{build_app, default_state};

#[actix_web::test]
async fn test_initialize() {
    let app = test::init_service(build_app(default_state())).await;

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "curl-test", "version": "0.1.0" }
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["result"]["protocolVersion"], "2025-06-18");
    assert!(body["result"]["capabilities"].is_object());
}


#[actix_web::test]
async fn test_unknown_tool_returns_error_payload() {
    let app = test::init_service(build_app(default_state())).await;

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "whatever_tool_does_not_exist",
                "arguments": { "input": "whatever" }
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK); // transport succeeded

    let body: serde_json::Value = test::read_body_json(resp).await;
    let is_error = body["result"]["isError"].as_bool().unwrap_or(false);
    assert!(is_error, "expected isError: true for unknown tool, got: {body}");
}

#[actix_web::test]
async fn test_eurlex_url_tool() {
    let app = test::init_service(build_app(default_state())).await;

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(serde_json::json!(
        {"jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
            "name": "sparql_reg_search",
                "arguments": {
                    "input": "data protection"
                }
            }
        }
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}