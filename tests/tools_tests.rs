//! Integration tests for the Ariadne MCP server (`POST /mcp`).
//!
//! Layout:
//!   1. Constants
//!   2. Request builders      – build JSON-RPC / MCP requests
//!   3. Call helpers          – send a request, check the envelope, return the body
//!   4. Assertion helpers     – reusable checks on tool results
//!   5. Protocol tests        – initialize, unknown tool
//!   6. Cellar tests          – EU legal documents
//!   7. TED tests             – EU procurement notices
//!   8. EDGAR tests           – SEC filings
//!   9. Rate limiter tests
//!
//! Note: most tests below hit live upstream services (Cellar, TED, EDGAR),
//! so they need network access.

use actix_web::{http::StatusCode, test, web};
use ariadne::utils::rate_limit::RateLimiter;
use ariadne::{build_app, default_limiter, default_state};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// 1. Constants
// ---------------------------------------------------------------------------

const MCP_PATH: &str = "/mcp";
const PROTOCOL_VERSION: &str = "2025-06-18";

/// Cellar base URI of the GDPR (Regulation (EU) 2016/679); manifestations
/// are addressed by appending a suffix such as `.0006.03`.
const GDPR_CELLAR_URI: &str =
    "http://publications.europa.eu/resource/cellar/3e485e15-11bd-11e6-ba9a-01aa75ed71a1";

/// Matches "Article <n>" plus up to 200 following chars.
const ARTICLE_REGEX: &str = r"Article \d+.{0,200}";

/// Upper bound for a regex-search result (e.g. MAX_MATCHES * MAX_MATCH_LEN).
const MAX_MANIFESTATION_RESULT_LEN: usize = 10_000;

/// Apple Inc. in EDGAR, as a plain and as a zero-padded (10-digit) CIK.
const APPLE_CIK: &str = "320193";
const APPLE_CIK_PADDED: &str = "0000320193";

/// Shared TED search fragment: German buyers mentioning "artificial intelligence".
const TED_AI_GERMANY: &str = "FT~\"artificial intelligence\" AND buyer-country=DEU";

// ---------------------------------------------------------------------------
// 2. Request builders
// ---------------------------------------------------------------------------

/// Builds a JSON-RPC 2.0 request to `/mcp` for the given method and params.
fn mcp_request(id: i64, method: &str, params: Value) -> test::TestRequest {
    test::TestRequest::post().uri(MCP_PATH).set_json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    }))
}

/// Builds a `tools/call` request for `tool` with the given `arguments`.
fn tool_request(id: i64, tool: &str, arguments: Value) -> test::TestRequest {
    mcp_request(
        id,
        "tools/call",
        json!({ "name": tool, "arguments": arguments }),
    )
}

/// Params for an `initialize` request, identifying the client as `client_name`.
fn initialize_params(client_name: &str) -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {},
        "clientInfo": { "name": client_name, "version": "0.1.0" }
    })
}

// ---------------------------------------------------------------------------
// 3. Call helpers
// ---------------------------------------------------------------------------

/// Sends one JSON-RPC request to a fresh app with default state and limiter.
///
/// Asserts the transport-level contract shared by every test: HTTP 200, and
/// a JSON-RPC 2.0 envelope that echoes the request `id`. Returns the body.
async fn rpc(id: i64, method: &str, params: Value) -> Value {
    let app = test::init_service(build_app(default_state(), default_limiter())).await;
    let resp = test::call_service(&app, mcp_request(id, method, params).to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], id);
    body
}

/// Like [`rpc`], for `tools/call`.
async fn call_tool(id: i64, tool: &str, arguments: Value) -> Value {
    rpc(id, "tools/call", json!({ "name": tool, "arguments": arguments })).await
}

// ---------------------------------------------------------------------------
// 4. Assertion helpers
// ---------------------------------------------------------------------------

/// Whether the tool result is flagged `isError: true` (missing flag = false).
fn is_tool_error(body: &Value) -> bool {
    body["result"]["isError"].as_bool().unwrap_or(false)
}

/// Asserts the call produced neither a JSON-RPC `error` nor `isError: true`.
fn assert_tool_ok(body: &Value, what: &str) {
    assert!(
        body.get("error").is_none(),
        "{what} returned a JSON-RPC error: {body:#}"
    );
    assert!(!is_tool_error(body), "{what} returned a tool error: {body}");
}

/// Asserts the call succeeded and returns its non-empty text payload
/// (`result.content[0].text`).
fn expect_tool_text<'a>(body: &'a Value, what: &str) -> &'a str {
    assert_tool_ok(body, what);
    let text = body["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("{what}: expected result.content[0].text to be a string, got: {body}"));
    assert!(!text.is_empty(), "{what}: expected non-empty text");
    text
}

/// Asserts `text` contains every string in `needles`.
fn assert_contains_all(text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(text.contains(needle), "expected result to contain {needle:?}");
    }
}

/// Runs `cellar.get_manifestation_content_with_regex` against one GDPR
/// manifestation and checks that it returns a non-empty result that stays
/// within [`MAX_MANIFESTATION_RESULT_LEN`] characters.
///
/// `manifestation` is the suffix appended to [`GDPR_CELLAR_URI`], and
/// `accept` the content type to request (XHTML, PDF/A, Formex 4...).
async fn assert_gdpr_manifestation_search(id: i64, manifestation: &str, accept: &str) {
    let body = call_tool(
        id,
        "cellar.get_manifestation_content_with_regex",
        json!({
            "url": format!("{GDPR_CELLAR_URI}{manifestation}"),
            "accept": accept,
            "language": "en",
            "regex_pattern": ARTICLE_REGEX,
        }),
    )
    .await;

    let text = expect_tool_text(&body, &format!("manifestation search ({accept})"));
    assert!(
        text.len() <= MAX_MANIFESTATION_RESULT_LEN,
        "expected search result for {accept} to stay within {MAX_MANIFESTATION_RESULT_LEN} chars, got {}",
        text.len()
    );
}

// ---------------------------------------------------------------------------
// 5. Protocol tests
// ---------------------------------------------------------------------------

/// `initialize` echoes the requested protocol version and advertises
/// server capabilities as an object.
#[actix_web::test]
async fn test_initialize() {
    let body = rpc(1, "initialize", initialize_params("curl-test")).await;

    assert_eq!(body["result"]["protocolVersion"], PROTOCOL_VERSION);
    assert!(body["result"]["capabilities"].is_object());
}

/// Calling a tool that doesn't exist is reported *inside* the result
/// (`isError: true`) rather than as an HTTP or JSON-RPC failure.
#[actix_web::test]
async fn test_unknown_tool_returns_error_payload() {
    let body = call_tool(
        4,
        "whatever_tool_does_not_exist",
        json!({ "input": "whatever" }),
    )
    .await; // `rpc` already asserted the transport succeeded

    assert!(
        is_tool_error(&body),
        "expected isError: true for unknown tool, got: {body}"
    );
}

// ---------------------------------------------------------------------------
// 6. Cellar tests
// ---------------------------------------------------------------------------

/// Keyword search on Cellar answers with HTTP 200.
///
/// The body is deliberately not read: the payload is large and we only care
/// that the request is handled, so we skip `rpc` and drop the response.
#[actix_web::test]
async fn test_tool_cellar_get_works_based_on_keyword() {
    let app = test::init_service(build_app(default_state(), default_limiter())).await;
    let req = tool_request(
        4,
        "cellar.get_works_based_on_keyword",
        json!({ "keyword": "data protection" }),
    );

    let resp = test::call_service(&app, req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

/// The `ted://search-guide` resource is readable and returns exactly one
/// non-empty markdown entry for that URI.
#[actix_web::test]
async fn test_resource_ted_search_guide() {
    let body = rpc(5, "resources/read", json!({ "uri": "ted://search-guide" })).await;

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

/// Regex search over the GDPR's XHTML manifestation returns a bounded,
/// non-empty result.
#[actix_web::test]
async fn test_tool_search_gdpr_xhtml() {
    assert_gdpr_manifestation_search(7, ".0006.03", "application/xhtml+xml").await;
}

/// Same regex search against the GDPR's PDF/A manifestation.
#[actix_web::test]
async fn test_tool_search_gdpr_pdf() {
    assert_gdpr_manifestation_search(8, ".0006.01", "application/pdf;type=pdfa1a").await;
}

/// Same regex search against the GDPR's Formex 4 (XML) manifestation.
#[actix_web::test]
async fn test_tool_search_gdpr_fmx4() {
    assert_gdpr_manifestation_search(9, ".0006.02/DOC_2", "application/xml;type=fmx4").await;
}

// ---------------------------------------------------------------------------
// 7. TED tests
// ---------------------------------------------------------------------------

/// `ted.search_notices` accepts an expert-syntax query (AI-related German
/// notices published since 2026) and succeeds.
#[actix_web::test]
async fn test_tool_ted_search() {
    let query = format!("{TED_AI_GERMANY} AND publication-date>=20260101");
    let body = call_tool(6, "ted.search_notices", json!({ "query": query })).await;

    assert_tool_ok(&body, "TED search");
}

/// `ted.search_awards` accepts a query restricted to contract-award notice
/// types (AI-related, German buyers, since 2026) and succeeds.
#[actix_web::test]
async fn test_tool_ted_award() {
    let query = format!(
        "{TED_AI_GERMANY} AND notice-type IN (can-standard can-social can-desg) \
         AND publication-date>=20260101"
    );
    let body = call_tool(7, "ted.search_awards", json!({ "query": query })).await;

    assert_tool_ok(&body, "TED award search");
}

// ---------------------------------------------------------------------------
// 8. EDGAR tests
// ---------------------------------------------------------------------------

/// `edgar.get_cik` resolves the ticker `AAPL` to Apple's zero-padded CIK.
#[actix_web::test]
async fn test_tool_edgar_get_cik() {
    let body = call_tool(8, "edgar.get_cik", json!({ "ticker": "AAPL" })).await;

    assert_eq!(expect_tool_text(&body, "EDGAR CIK lookup"), APPLE_CIK_PADDED);
}

/// `edgar.get_company_tags` lists Apple's reported XBRL concepts, including
/// the `us-gaap` taxonomy and the `Assets` tag.
#[actix_web::test]
async fn test_tool_edgar_company_tags() {
    let body = call_tool(7, "edgar.get_company_tags", json!({ "cik": APPLE_CIK })).await;

    let text = expect_tool_text(&body, "EDGAR company tags");
    assert_contains_all(text, &[APPLE_CIK_PADDED, "Apple Inc.", "us-gaap", "Assets"]);
}

/// `edgar.get_company_concept` returns Apple's `us-gaap:Assets` facts,
/// including the concept's `label` and `description` metadata.
#[actix_web::test]
async fn test_tool_edgar_company_concept() {
    let body = call_tool(
        8,
        "edgar.get_company_concept",
        json!({ "cik": APPLE_CIK, "taxonomy": "us-gaap", "tag": "Assets" }),
    )
    .await;

    let text = expect_tool_text(&body, "EDGAR company concept");
    assert_contains_all(
        text,
        &[APPLE_CIK_PADDED, "Apple Inc.", "us-gaap", "Assets", "label", "description"],
    );
}

// ---------------------------------------------------------------------------
// 9. Rate limiter tests
// ---------------------------------------------------------------------------

/// Builds an `initialize` request as if sent from `ip` (port `12345` is
/// appended); the rate limiter keys its buckets on this peer address.
///
/// `initialize` is the cheapest valid MCP request and needs no session
/// state, so any non-200 in these tests is attributable to the limiter.
fn initialize_req(ip: &str) -> test::TestRequest {
    mcp_request(1, "initialize", initialize_params("rate-limit-test"))
        .peer_addr(format!("{ip}:12345").parse().unwrap())
}

/// A client is rejected once its burst allowance is used up.
///
/// Limiter: burst of 2, refill of 0.001 tokens/sec (negligible during the test).
/// 1. The first two requests from one IP succeed (`200 OK`).
/// 2. The third is rejected with `429 Too Many Requests`.
/// 3. The 429 carries a `Retry-After` header.
#[actix_web::test]
async fn test_rate_limit_blocks_after_burst() {
    let limiter = web::Data::new(RateLimiter::new(2, 0.001));
    let app = test::init_service(build_app(default_state(), limiter)).await;

    for _ in 0..2 {
        let resp = test::call_service(&app, initialize_req("203.0.113.1").to_request()).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    let resp = test::call_service(&app, initialize_req("203.0.113.1").to_request()).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(resp.headers().contains_key("retry-after"));
}

/// Rate limits are tracked per client IP: one client exhausting its
/// allowance must not affect another.
///
/// Limiter: burst of 1, negligible refill.
/// 1. `203.0.113.1` uses its single token (`200 OK`), then gets `429`.
/// 2. `203.0.113.2` still succeeds on its first request (own bucket).
///
/// (`203.0.113.0/24` is TEST-NET-3, reserved for documentation, so it is
/// safe to use in tests.)
#[actix_web::test]
async fn test_rate_limit_is_per_ip() {
    let limiter = web::Data::new(RateLimiter::new(1, 0.001));
    let app = test::init_service(build_app(default_state(), limiter)).await;

    // IP A uses its single token, then is blocked
    let resp = test::call_service(&app, initialize_req("203.0.113.1").to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = test::call_service(&app, initialize_req("203.0.113.1").to_request()).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    // IP B is unaffected
    let resp = test::call_service(&app, initialize_req("203.0.113.2").to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);
}