use serde_json::json;

use crate::utils::{Client, ToolError};

pub async fn ted_search(
    client: &Client,
    input: &str,
) -> Result<String, ToolError> {
    let body = json!({
        "query": input,
        "fields": [
            "publication-number",
            "publication-date",
            "notice-type",
            "notice-title",
            "description-proc",
            "classification-cpv",
            "buyer-name",
            "buyer-country",
            "place-of-performance-country-proc",
            "contract-nature-main-proc",
            "estimated-value-cur-proc",
            "deadline-receipt-tender-date-lot",
            "procedure-identifier"
        ],
        "limit": 10,
        "scope": "ACTIVE",
        "paginationMode": "ITERATION"
    });

    client
        .post_json(
            "https://api.ted.europa.eu/v3/notices/search",
            &body,
        )
        .await
        .map_err(|e| ToolError::Http(e.to_string()))
}

pub async fn ted_award(
    client: &Client,
    input: &str,
) -> Result<String, ToolError> {
    let body = json!({
        "query": input,
        "fields": [
            "publication-number",
            "publication-date",
            "notice-title",
            "buyer-name",
            "buyer-country",
            "classification-cpv",
            "winner-identifier",
            "winner-partname",
            "winner-country",
            "result-value-notice",
            "result-value-cur-notice",
            "contract-conclusion-date"
        ],
        "limit": 10,
        "scope": "ACTIVE",
        "paginationMode": "ITERATION"
    });

    client
        .post_json(
            "https://api.ted.europa.eu/v3/notices/search",
            &body,
        )
        .await
        .map_err(|e| ToolError::Http(e.to_string()))
}