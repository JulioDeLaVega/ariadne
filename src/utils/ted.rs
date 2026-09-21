use serde_json::{json, Value};
use crate::utils::{Client, ToolError};

const TED_SEARCH_URL: &str = "https://api.ted.europa.eu/v3/notices/search";

fn remove_links(response: &str) -> Result<String, ToolError> {
    let mut data: Value = serde_json::from_str(response)
        .map_err(|e| ToolError::Http(e.to_string()))?;

    if let Some(notices) = data
        .get_mut("notices")
        .and_then(|v| v.as_array_mut())
    {
        for notice in notices {
            if let Some(obj) = notice.as_object_mut() {
                obj.remove("links");
            }
        }
    }

    serde_json::to_string(&data)
        .map_err(|e| ToolError::Http(e.to_string()))
}

pub async fn search_notices(
    client: &Client,
    arguments: &serde_json::Value,
) -> Result<String, ToolError> {
    let body = json!({
        "query": arguments.get("query").and_then(|v| v.as_str()).unwrap_or(""),
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
        "limit": 50,
        "scope": "ACTIVE",
        "paginationMode": "ITERATION"
    });

    let response = client
        .post_json(TED_SEARCH_URL, &body)
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    remove_links(&response)
}

pub async fn search_awards(
    client: &Client,
    arguments: &serde_json::Value,
) -> Result<String, ToolError> {
    let body = json!({
        "query": arguments.get("query").and_then(|v| v.as_str()).unwrap_or(""),
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
        "limit": 50,
        "scope": "ACTIVE",
        "paginationMode": "ITERATION"
    });

    let response = client
        .post_json(TED_SEARCH_URL, &body)
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    remove_links(&response)
}