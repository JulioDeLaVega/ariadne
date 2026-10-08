use crate::utils::{ToolError, AppState};

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

const SEC_BASE_CIK: &str = "https://www.sec.gov";
const SEC_BASE_COMPANYFACTS: &str = "https://data.sec.gov/api/xbrl/companyfacts";
const SEC_BASE_COMPANYCONCEPT: &str = "https://data.sec.gov/api/xbrl/companyconcept";


static TICKER_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

pub async fn helper_get_cik(
    state: &AppState,
    arguments: &Value,
) -> Result<String, ToolError> {

    let ticker = arguments
        .get("ticker")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if ticker.is_empty() {
        return Err(ToolError::MissingInput);
    }

    // Return from cache if already initialized.
    if let Some(cache) = TICKER_CACHE.get() {
        return cache
            .get(&ticker.to_uppercase())
            .cloned()
            .ok_or(ToolError::InvalidInput("Invalid input".into()));
    }

    let mut headers = HeaderMap::new();

    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&state.config.email).map_err(|e| ToolError::Http(e.to_string()))?,
    );

    let url = format!("{SEC_BASE_CIK}/files/company_tickers.json");

    let bytes = state.client
        .get_bytes(&url, headers)
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    let data: Value = serde_json::from_slice(&bytes)
        .map_err(|e| ToolError::Http(e.to_string()))?;

    let mut cache = HashMap::new();

    if let Some(companies) = data.as_object() {
        for company in companies.values() {
            let Some(ticker) = company["ticker"].as_str() else {
                continue;
            };

            let Some(cik) = company["cik_str"].as_u64() else {
                continue;
            };

            cache.insert(
                ticker.to_uppercase(),
                format!("{cik:010}"),
            );
        }
    }

    let _ = TICKER_CACHE.set(cache);

    TICKER_CACHE
        .get()
        .and_then(|cache| {
            cache.get(&ticker.to_uppercase()).cloned()
        })
        .ok_or(ToolError::InvalidInput("Invalid input".into()))
}

use std::collections::BTreeMap;

// Only the fields we need; serde skips everything else.
#[derive(Deserialize)]
struct CompanyFacts {
    #[serde(rename = "entityName", default)]
    entity_name: String,
    facts: BTreeMap<String, BTreeMap<String, Concept>>, // taxonomy -> tag -> concept
}

#[derive(Deserialize)]
struct Concept {
    label: Option<String>,
    units: BTreeMap<String, Vec<Fact>>, // unit -> facts
}

#[derive(Deserialize)]
struct Fact {
    end: String, // ISO date, so string comparison orders correctly
}

pub async fn helper_get_structure(
    state: &AppState,
    arguments: &Value,
) -> Result<String, ToolError> {
    // --- input ---------------------------------------------------------
    let cik = match arguments.get("cik") {
        Some(Value::String(s)) => s.trim().trim_start_matches("CIK").to_string(),
        Some(Value::Number(n)) => n.to_string(),
        _ => return Err(ToolError::MissingInput),
    };

    if cik.is_empty() {
        return Err(ToolError::MissingInput);
    }
    if !cik.chars().all(|c| c.is_ascii_digit()) || cik.len() > 10 {
        return Err(ToolError::InvalidInput("Invalid input".into()));
    }

    let cik = format!("{cik:0>10}");
    let url = format!("{SEC_BASE_COMPANYFACTS}/CIK{cik}.json");

    // --- headers -------------------------------------------------------

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&state.config.email).map_err(|e| ToolError::Http(e.to_string()))?,
    );

    // --- fetch + parse -------------------------------------------------
    let bytes = state.client
        .get_bytes(&url, headers)
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    let data: CompanyFacts = serde_json::from_slice(&bytes)
        .map_err(|e| ToolError::Http(format!("failed to parse companyfacts: {e}")))?;

    // --- build catalog -------------------------------------------------
    let taxonomies: BTreeMap<&String, usize> =
        data.facts.iter().map(|(t, tags)| (t, tags.len())).collect();

    let mut rows: Vec<Value> = Vec::new();

    for (taxonomy, tags) in &data.facts {
        for (tag, concept) in tags {
            let units: Vec<&String> = concept.units.keys().collect();

            let mut n_facts = 0usize;
            let mut first_end: Option<&str> = None;
            let mut latest_end: Option<&str> = None;

            for f in concept.units.values().flatten() {
                n_facts += 1;
                let end = f.end.as_str();
                if first_end.map_or(true, |m| end < m) {
                    first_end = Some(end);
                }
                if latest_end.map_or(true, |m| end > m) {
                    latest_end = Some(end);
                }
            }

            rows.push(json!([
                taxonomy,
                tag,
                concept.label,
                units,
                n_facts,
                first_end,
                latest_end,
            ]));
        }
    }

    let result = json!({
        "cik": cik,
        "entity_name": data.entity_name,
        "source_url": url,
        "taxonomies": taxonomies,
        "total_rows": rows.len(),
        "columns": [
            "taxonomy", "tag", "label", "units",
            "n_facts", "first_end", "latest_end"
        ],
        "rows": rows,
    });

    serde_json::to_string(&result).map_err(|e| ToolError::Http(e.to_string()))
}

pub fn trim_output(
    value: &mut Value,
    max_chars: usize,
) {
    while serialized_len(value) > max_chars {
        match value {
            Value::Array(array) if !array.is_empty() => {
                array.remove(0);
            }
            _ => break,
        }
    }
}

fn serialized_len(value: &Value) -> usize {
    serde_json::to_string(value)
        .map(|s| s.len())
        .unwrap_or(usize::MAX)
}

pub async fn xbrl_get_company_concept(state: &AppState, arguments: &Value) -> Result<String, ToolError> {
    // --- input ---------------------------------------------------------

    let cik = match arguments.get("cik") {
        Some(Value::String(s)) => {
            s.trim().trim_start_matches("CIK").to_string()
        }
        Some(Value::Number(n)) => n.to_string(),
        _ => return Err(ToolError::MissingInput),
    };

    if cik.is_empty() {
        return Err(ToolError::MissingInput);
    }

    if !cik.chars().all(|c| c.is_ascii_digit()) || cik.len() > 10 {
        return Err(ToolError::InvalidInput("Invalid input".into()));
    }

    let taxonomy = match arguments.get("taxonomy") {
        Some(Value::String(s)) if !s.trim().is_empty() => {
            s.trim().to_string()
        }
        _ => return Err(ToolError::MissingInput),
    };

    let tag = match arguments.get("tag") {
        Some(Value::String(s)) if !s.trim().is_empty() => {
            s.trim().to_string()
        }
        _ => return Err(ToolError::MissingInput),
    };

    let cik = format!("{cik:0>10}");

    // --- URL -----------------------------------------------------------

    let url = format!(
        "{SEC_BASE_COMPANYCONCEPT}/CIK{cik}/{taxonomy}/{tag}.json"
    );

    // --- headers -------------------------------------------------------

    let mut headers = HeaderMap::new();

    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&state.config.email)
            .map_err(|e| ToolError::Http(e.to_string()))?,
    );

    // --- fetch ---------------------------------------------------------

    let bytes = state.client
        .get_bytes(&url, headers)
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    // --- parse ---------------------------------------------------------

    let data: Value = serde_json::from_slice(&bytes)
        .map_err(|e| {
            ToolError::Http(format!(
                "failed to parse companyconcept: {e}"
            ))
        })?;

    // --- build result --------------------------------------------------

    let mut result = json!({
        "cik": cik,
        "taxonomy": taxonomy,
        "tag": tag,
        "entity_name": data.get("entityName"),
        "label": data.get("label"),
        "description": data.get("description"),
        "units": data.get("units"),
        "source_url": url,
    });

    // --- trim historical facts ----------------------------------------

    const MAX_OUTPUT_CHARS: usize = 40_000;

    if let Some(units) = result
        .get_mut("units")
        .and_then(Value::as_object_mut)
    {
        for facts in units.values_mut() {
            trim_output(facts, MAX_OUTPUT_CHARS);
        }
    }

    // --- output --------------------------------------------------------

    serde_json::to_string(&result)
        .map_err(|e| ToolError::Http(e.to_string()))
}

const SEC_BASE_FRAMES: &str = "https://data.sec.gov/api/xbrl/frames";

pub async fn xbrl_get_frame(
    state: &AppState,
    arguments: &Value,
) -> Result<String, ToolError> {
    // --- required input -----------------------------------------------

    let taxonomy = match arguments.get("taxonomy") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return Err(ToolError::MissingInput),
    };

    let tag = match arguments.get("tag") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return Err(ToolError::MissingInput),
    };

    let unit = match arguments.get("unit") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return Err(ToolError::MissingInput),
    };

    let frame = match arguments.get("frame") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return Err(ToolError::MissingInput),
    };

    // --- URL -----------------------------------------------------------

    let url = format!(
        "{SEC_BASE_FRAMES}/{taxonomy}/{tag}/{unit}/{frame}.json"
    );

    // --- headers -------------------------------------------------------

    let mut headers = HeaderMap::new();

    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&state.config.email)
            .map_err(|e| ToolError::Http(e.to_string()))?,
    );

    // --- fetch ---------------------------------------------------------

    let bytes = state
        .client
        .get_bytes(&url, headers)
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    // --- parse ---------------------------------------------------------

    let mut data: Value = serde_json::from_slice(&bytes).map_err(|e| {
        ToolError::Http(format!("Failed to parse XBRL frame: {e}"))
    })?;

    // --- apply filters -------------------------------------------------

    filter_frame_data(&mut data, arguments)?;

    // --- output --------------------------------------------------------

    let mut result = json!({
        "taxonomy": taxonomy,
        "tag": tag,
        "unit": unit,
        "frame": frame,
        "label": data.get("label"),
        "description": data.get("description"),
        "entity_name": data.get("entityName"),
        "data": data.get("data"),
        "source_url": url,
    });

    // --- trim large result ---------------------------------------------

    const MAX_OUTPUT_CHARS: usize = 40_000;

    if let Some(data) = result.get_mut("data") {
        trim_output(data, MAX_OUTPUT_CHARS);
    }

    // --- serialize -----------------------------------------------------

    serde_json::to_string(&result)
        .map_err(|e| ToolError::Http(e.to_string()))
}

pub fn filter_frame_data(
    data: &mut Value,
    arguments: &Value,
) -> Result<(), ToolError> {
    let cik = arguments.get("cik").and_then(Value::as_u64);

    let accn = arguments
        .get("accn")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let entity_name = arguments
        .get("entity_name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase);

    let loc = arguments
        .get("loc")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let val_filter = arguments
        .get("val")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let val_filter = if let Some(filter) = val_filter {
        let (operator, value_str) =
            if let Some(value) = filter.strip_suffix('+') {
                ('+', value.trim())
            } else if let Some(value) = filter.strip_suffix('-') {
                ('-', value.trim())
            } else {
                return Err(ToolError::InvalidInput("Invalid val filter. Use '<number>+' or '<number>-'.".into()));
            };

        let threshold: f64 = value_str.parse().map_err(|_| {
            ToolError::InvalidInput("Invalid val threshold. Expected a number followed by '+' or '-'.".into())
        })?;

        if !threshold.is_finite() {
            return Err(ToolError::InvalidInput("The val threshold must be a finite number.".into()));
        }

        Some((operator, threshold))
    } else {
        None
    };

    if let Some(records) = data.get_mut("data").and_then(Value::as_array_mut) {
        records.retain(|record| {
            if let Some(filter) = cik {
                if record.get("cik").and_then(Value::as_u64) != Some(filter) {
                    return false;
                }
            }

            if let Some(ref filter) = accn {
                if record.get("accn").and_then(Value::as_str)
                    != Some(filter.as_str())
                {
                    return false;
                }
            }

            if let Some(ref filter) = entity_name {
                let matches = record
                    .get("entityName")
                    .and_then(Value::as_str)
                    .map(|name| name.to_lowercase().contains(filter))
                    .unwrap_or(false);

                if !matches {
                    return false;
                }
            }

            if let Some(ref filter) = loc {
                if record.get("loc").and_then(Value::as_str)
                    != Some(filter.as_str())
                {
                    return false;
                }
            }

            if let Some((operator, threshold)) = val_filter {
                let record_value = match record.get("val").and_then(Value::as_f64) {
                    Some(value) => value,
                    None => return false,
                };

                match operator {
                    '+' if record_value < threshold => return false,
                    '-' if record_value > threshold => return false,
                    _ => {}
                }
            }

            true
        });
    }

    Ok(())
}