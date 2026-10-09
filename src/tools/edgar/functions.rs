use crate::utils::{ToolError, AppState};

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

const SEC_WWW_CIK: &str = "https://www.sec.gov";
const SEC_DATA_COMPANYFACTS: &str = "https://data.sec.gov/api/xbrl/companyfacts";
const SEC_DATA_COMPANYCONCEPT: &str = "https://data.sec.gov/api/xbrl/companyconcept";

const SEC_BASE_SUBMISSIONS: &str = "https://data.sec.gov/submissions";
const SEC_BASE_ARCHIVES: &str = "https://www.sec.gov/Archives/edgar/data";


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

    let url = format!("{SEC_WWW_CIK}/files/company_tickers.json");

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
    let url = format!("{SEC_DATA_COMPANYFACTS}/CIK{cik}.json");

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
        "{SEC_DATA_COMPANYCONCEPT}/CIK{cik}/{taxonomy}/{tag}.json"
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

const SEC_DATA_FRAMES: &str = "https://data.sec.gov/api/xbrl/frames";

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
        "{SEC_DATA_FRAMES}/{taxonomy}/{tag}/{unit}/{frame}.json"
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



// Assumes the same imports/helpers as `xbrl_get_frame`:
//   AppState, ToolError, HeaderMap, HeaderValue, USER_AGENT, trim_output,
//   serde_json::{json, Value}

/// Parse an optional YYYY-MM-DD argument. ISO dates compare correctly as strings.
fn optional_date_arg(arguments: &Value, key: &str) -> Result<Option<String>, ToolError> {
    match arguments.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if s.trim().is_empty() => Ok(None),
        Some(Value::String(s)) => {
            let s = s.trim();
            let b = s.as_bytes();
            let ok = b.len() == 10
                && b[4] == b'-'
                && b[7] == b'-'
                && b.iter()
                    .enumerate()
                    .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
            if ok {
                Ok(Some(s.to_string()))
            } else {
                Err(ToolError::Http(format!(
                    "Invalid {key}: expected YYYY-MM-DD, got '{s}'"
                )))
            }
        }
        _ => Err(ToolError::MissingInput),
    }
}

/// `form` may be a string ("10-Q"), a comma-separated string ("10-Q,10-K"),
/// or an array of strings. Returns upper-cased forms; empty = no filter.
fn forms_arg(arguments: &Value) -> Vec<String> {
    match arguments.get("form") {
        Some(Value::String(s)) => s
            .split(',')
            .map(|f| f.trim().to_uppercase())
            .filter(|f| !f.is_empty())
            .collect(),
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str())
            .map(|f| f.trim().to_uppercase())
            .filter(|f| !f.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Convert a columnar block ({"form": [...], "filingDate": [...], ...})
/// into row objects, applying the filters. Rows keep the source order
/// (SEC returns newest first).
fn collect_filings(
    block: &Value,
    cik: &str,
    forms: &[String],
    from: Option<&str>,
    to: Option<&str>,
    out: &mut Vec<Value>,
) {
    let col = |name: &str| block.get(name).and_then(|v| v.as_array());

    let (Some(dates), Some(form_col), Some(accs)) =
        (col("filingDate"), col("form"), col("accessionNumber"))
    else {
        return;
    };

    let report_dates = col("reportDate");
    let prim_docs = col("primaryDocument");
    let prim_desc = col("primaryDocDescription");
    let items = col("items");
    let sizes = col("size");
    let acc_times = col("acceptanceDateTime");
    let is_xbrl = col("isXBRL");

    let str_at = |c: Option<&Vec<Value>>, i: usize| -> String {
        c.and_then(|a| a.get(i))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };

    let cik_trimmed = cik.trim_start_matches('0');

    for i in 0..dates.len() {
        let date = dates[i].as_str().unwrap_or("");
        let form = form_col.get(i).and_then(|v| v.as_str()).unwrap_or("");

        if let Some(f) = from {
            if date < f {
                continue;
            }
        }
        if let Some(t) = to {
            if date > t {
                continue;
            }
        }
        if !forms.is_empty() && !forms.iter().any(|f| f.eq_ignore_ascii_case(form)) {
            continue;
        }

        let acc = accs.get(i).and_then(|v| v.as_str()).unwrap_or("");
        let acc_nodash = acc.replace('-', "");
        let doc = str_at(prim_docs, i);

        let url = if doc.is_empty() {
            format!("{SEC_BASE_ARCHIVES}/{cik_trimmed}/{acc_nodash}/")
        } else {
            format!("{SEC_BASE_ARCHIVES}/{cik_trimmed}/{acc_nodash}/{doc}")
        };

        out.push(json!({
            "form": form,
            "filing_date": date,
            "report_date": str_at(report_dates, i),
            "accession_number": acc,
            "acceptance_datetime": str_at(acc_times, i),
            "primary_document": doc,
            "primary_doc_description": str_at(prim_desc, i),
            "items": str_at(items, i),
            "size": sizes.and_then(|a| a.get(i)).cloned().unwrap_or(Value::Null),
            "is_xbrl": is_xbrl.and_then(|a| a.get(i)).cloned().unwrap_or(Value::Null),
            "url": url,
        }));
    }
}

pub async fn get_submissions(
    state: &AppState,
    arguments: &Value,
) -> Result<String, ToolError> {
    // --- required input -----------------------------------------------

    let cik_raw = match arguments.get("cik") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
        Some(Value::Number(n)) => n.to_string(),
        _ => return Err(ToolError::MissingInput),
    };

    // Accept "1045810", "0001045810", "CIK0001045810".
    let digits: String = cik_raw
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .to_string();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) || digits.len() > 10 {
        return Err(ToolError::Http(format!("Invalid CIK: '{cik_raw}'")));
    }
    let cik = format!("{:0>10}", digits);

    // --- optional filters ---------------------------------------------

    let forms = forms_arg(arguments);
    let from = optional_date_arg(arguments, "filing_date_from")?;
    let to = optional_date_arg(arguments, "filing_date_to")?;

    if let (Some(f), Some(t)) = (&from, &to) {
        if f > t {
            return Err(ToolError::Http(
                "filing_date_from must not be after filing_date_to".to_string(),
            ));
        }
    }

    let limit = arguments
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(100)
        .clamp(1, 1000);

    // --- headers -------------------------------------------------------

    let make_headers = || -> Result<HeaderMap, ToolError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&state.config.email)
                .map_err(|e| ToolError::Http(e.to_string()))?,
        );
        Ok(headers)
    };

    // --- fetch main submissions file -----------------------------------

    let url = format!("{SEC_BASE_SUBMISSIONS}/CIK{cik}.json");

    let bytes = state
        .client
        .get_bytes(&url, make_headers()?)
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    let data: Value = serde_json::from_slice(&bytes)
        .map_err(|e| ToolError::Http(format!("Failed to parse submissions: {e}")))?;

    // --- collect filings from "recent" ---------------------------------

    let mut filings: Vec<Value> = Vec::new();

    if let Some(recent) = data.get("filings").and_then(|f| f.get("recent")) {
        collect_filings(recent, &cik, &forms, from.as_deref(), to.as_deref(), &mut filings);
    }

    // --- older filings live in additional files ------------------------
    // Each entry: {"name": "...", "filingFrom": "YYYY-MM-DD", "filingTo": "YYYY-MM-DD"}.
    // Only fetch the ones whose range overlaps the requested date window.

    let mut extra_files_fetched: Vec<String> = Vec::new();

    if let Some(files) = data
        .get("filings")
        .and_then(|f| f.get("files"))
        .and_then(|f| f.as_array())
    {
        for f in files {
            let (Some(name), Some(f_from), Some(f_to)) = (
                f.get("name").and_then(|v| v.as_str()),
                f.get("filingFrom").and_then(|v| v.as_str()),
                f.get("filingTo").and_then(|v| v.as_str()),
            ) else {
                continue;
            };

            // Skip if the file's range lies entirely outside the window.
            if let Some(from) = &from {
                if f_to < from.as_str() {
                    continue;
                }
            }
            if let Some(to) = &to {
                if f_from > to.as_str() {
                    continue;
                }
            }

            let file_url = format!("{SEC_BASE_SUBMISSIONS}/{name}");

            let bytes = state
                .client
                .get_bytes(&file_url, make_headers()?)
                .await
                .map_err(|e| ToolError::Http(e.to_string()))?;

            // These files are a bare columnar block (no "filings" wrapper).
            let block: Value = serde_json::from_slice(&bytes).map_err(|e| {
                ToolError::Http(format!("Failed to parse submissions file {name}: {e}"))
            })?;

            collect_filings(&block, &cik, &forms, from.as_deref(), to.as_deref(), &mut filings);
            extra_files_fetched.push(file_url);
        }
    }

    // --- sort newest first, truncate -----------------------------------

    filings.sort_by(|a, b| {
        let da = a.get("filing_date").and_then(|v| v.as_str()).unwrap_or("");
        let db = b.get("filing_date").and_then(|v| v.as_str()).unwrap_or("");
        db.cmp(da).then_with(|| {
            let aa = a.get("accession_number").and_then(|v| v.as_str()).unwrap_or("");
            let ab = b.get("accession_number").and_then(|v| v.as_str()).unwrap_or("");
            ab.cmp(aa)
        })
    });

    let total_matches = filings.len();
    filings.truncate(limit);

    // --- output --------------------------------------------------------

    let mut result = json!({
        "cik": cik,
        "name": data.get("name"),
        "tickers": data.get("tickers"),
        "sic": data.get("sic"),
        "sic_description": data.get("sicDescription"),
        "fiscal_year_end": data.get("fiscalYearEnd"),
        "filters": {
            "form": forms,
            "filing_date_from": from,
            "filing_date_to": to,
            "limit": limit,
        },
        "total_matches": total_matches,
        "returned": filings.len(),
        "filings": filings,
        "source_url": url,
        "additional_files_fetched": extra_files_fetched,
    });

    // --- trim large result ---------------------------------------------

    const MAX_OUTPUT_CHARS: usize = 40_000;

    if let Some(f) = result.get_mut("filings") {
        trim_output(f, MAX_OUTPUT_CHARS);
    }

    // --- serialize -----------------------------------------------------

    serde_json::to_string(&result).map_err(|e| ToolError::Http(e.to_string()))
}