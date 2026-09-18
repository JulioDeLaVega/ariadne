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
async fn test_tool_eurlex_url() {
    let app = test::init_service(build_app(default_state())).await;

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "eurlex_url",
                "arguments": {
                    "input": "32016R0679"
                }
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    // deliberately not calling test::read_body(resp) — avoids materializing the large payload
}

#[actix_web::test]
async fn test_tool_sparql_reg_search() {
    let app = test::init_service(build_app(default_state())).await;

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "sparql_reg_search",
                "arguments": {
                    "input": "data protection"
                }
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    // deliberately not calling test::read_body(resp) — avoids materializing the large payload
}

#[actix_web::test]
async fn test_resource_ted_search_guide() {
    let app = test::init_service(build_app(default_state())).await;

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "resources/read",
            "params": {
                "uri": "ted://search-guide"
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 5);

    let contents = body["result"]["contents"]
        .as_array()
        .expect("expected result.contents to be an array");

    assert_eq!(contents.len(), 1);

    assert_eq!(contents[0]["uri"], "ted://search-guide");
    assert_eq!(contents[0]["mimeType"], "text/markdown");

    let text = contents[0]["text"]
        .as_str()
        .expect("expected resource text to be a string");

    assert!(!text.is_empty());

}

#[actix_web::test]
async fn test_tool_ted_search() {
    let app = test::init_service(build_app(default_state())).await;

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "ted.search_notices",
                "arguments": {
                    "input": "FT~\"artificial intelligence\" AND buyer-country=DEU AND publication-date>=20260101"
                }
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 6);

    let is_error = body["result"]["isError"].as_bool().unwrap_or(false);
    assert!(!is_error, "TED search returned an error: {body}");
}

#[actix_web::test]
async fn test_tool_ted_award() {
    let app = test::init_service(build_app(default_state())).await;

    let req = test::TestRequest::post()
        .uri("/mcp")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "ted.search_awards",
                "arguments": {
                    "input": "FT~\"artificial intelligence\" AND buyer-country=DEU AND notice-type IN (can-standard can-social can-desg) AND publication-date>=20260101"
                }
            }
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 7);

    let is_error = body["result"]["isError"].as_bool().unwrap_or(false);
    assert!(!is_error, "TED award search returned an error: {body}");
}