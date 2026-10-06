use crate::utils::{ToolError, AppState};
use reqwest::header::{HeaderMap, ACCEPT, ACCEPT_LANGUAGE};

const PREFIX: &str = "PREFIX cdm: <http://publications.europa.eu/ontology/cdm#>";

pub async fn get_works_based_on_keyword(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {
    let keyword = arguments.get("keyword").and_then(|v| v.as_str()).unwrap_or("");
    let query = format!(r#"{PREFIX}
        SELECT DISTINCT ?work ?celex ?title
        WHERE {{
        ?work cdm:work_has_resource-type <http://publications.europa.eu/resource/authority/resource-type/REG> .
        ?work cdm:resource_legal_id_celex ?celex .
        ?expr cdm:expression_belongs_to_work ?work .
        ?expr cdm:expression_title ?title .
        FILTER(CONTAINS(LCASE(STR(?title)), LCASE("{keyword}")))
        FILTER(lang(?title) = "en" || lang(?title) = "")
        }}
        LIMIT 3"#);
    
    state.client.sparql_query(&state.config.sparql_endpoint, &query).await.map_err(ToolError::from)

}

pub async fn get_predicates(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {
    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");
    let query = format!(r#"{PREFIX}
        SELECT DISTINCT ?predicate ?object
        WHERE {{
            <{uri}> ?predicate ?object .
        }}
        LIMIT 200"#);
    
    state.client.sparql_query(&state.config.sparql_endpoint, &query).await.map_err(ToolError::from)
}

pub async fn uri_from_celex(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {

    let celex = arguments.get("celex").and_then(|v| v.as_str()).unwrap_or("");
    let input = celex.to_lowercase();

    let query = format!(r#"{PREFIX}
        SELECT DISTINCT ?work ?celex
        WHERE {{
        ?work cdm:resource_legal_id_celex ?celex .
        FILTER(LCASE(STR(?celex)) = "{input}")
        }}
        LIMIT 3"#);
    
    state.client.sparql_query(&state.config.sparql_endpoint, &query).await.map_err(ToolError::from)

}

pub async fn get_objects_based_on_uri(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {
    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");
    let query = format!(r#"{PREFIX}
        SELECT DISTINCT ?subject ?celex ?type ?resourceType ?title ?date
        WHERE {{
            ?subject cdm:resource_legal_based_on_resource_legal <{uri}> .
            OPTIONAL {{ ?subject cdm:resource_legal_id_celex ?celex . }}
            OPTIONAL {{ ?subject cdm:resource_legal_type ?type . }}
            OPTIONAL {{ ?subject cdm:work_has_resource-type ?resourceType . }}
            OPTIONAL {{ ?subject cdm:work_title ?title . }}
            OPTIONAL {{ ?subject cdm:work_date_document ?date . }}
        }}
        LIMIT 200"#);
    
    state.client.sparql_query(&state.config.sparql_endpoint, &query).await.map_err(ToolError::from)
}

pub async fn get_resource_adopts_resource(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {
    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");
    let query = format!(r#"{PREFIX}
        SELECT ?object
        WHERE {{
        <{uri}> cdm:resource_legal_adopts_resource_legal ?object .
        }}
        LIMIT 2"#);

    state.client.sparql_query(&state.config.sparql_endpoint, &query).await.map_err(ToolError::from)
}

pub async fn get_resource_legal_information_miscellaneous(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {
    
    let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("");

    let query = format!(r#"{PREFIX}
        SELECT DISTINCT ?subject ?celex
        WHERE {{
            ?subject cdm:resource_legal_information_miscellaneous ?procedure .
            FILTER(CONTAINS(LCASE(STR(?procedure)), LCASE("{text}")))
            OPTIONAL {{
                ?subject cdm:resource_legal_id_celex ?celex .
            }}
        }}
        LIMIT 30"#);

    state.client.sparql_query(&state.config.sparql_endpoint, &query).await.map_err(ToolError::from)
}

pub async fn get_expressions_based_on_work(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {

    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");

    let query = format!(r#"{PREFIX}
        SELECT DISTINCT ?expression ?language ?title
        WHERE {{
            ?expression cdm:expression_belongs_to_work <{uri}> .
            OPTIONAL {{ ?expression cdm:expression_uses_language ?language . }}
            OPTIONAL {{ ?expression cdm:expression_title ?title . }}
        }}
        LIMIT 100"#);

    state.client.sparql_query(&state.config.sparql_endpoint, &query).await.map_err(ToolError::from)
}

pub async fn get_manifestations_based_on_expression(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {

    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");

    let query = format!(r#"{PREFIX}
        SELECT DISTINCT ?manifestation ?type ?item
        WHERE {{
            ?manifestation cdm:manifestation_manifests_expression <{uri}> .
            OPTIONAL {{ ?manifestation cdm:manifestation_type ?type . }}
            OPTIONAL {{ ?manifestation cdm:manifestation_has_item ?item . }}
        }}
        LIMIT 100"#);

    state.client.sparql_query(&state.config.sparql_endpoint, &query).await.map_err(ToolError::from)
}

use crate::utils::{parse_fmx4, parse_pdf, parse_xhtml};
use crate::utils::search_text;

pub async fn get_manifestation_content_with_regex(state: &AppState, arguments: &serde_json::Value) -> Result<String, ToolError> {
    let url = arguments.get("url").and_then(|v| v.as_str()).ok_or(ToolError::MissingInput)?;
    let format = arguments.get("accept").and_then(|v| v.as_str());       // Option<&str>
    let language = arguments.get("language").and_then(|v| v.as_str());   // Option<&str>
    let regex_pattern = arguments.get("regex_pattern").and_then(|v| v.as_str()).ok_or(ToolError::MissingInput)?;
    let max_matches = arguments.get("max_matches").and_then(|v| v.as_u64()).unwrap_or(2);

    let mut headers = HeaderMap::new();

    if let Some(value) = format {
        headers.insert(ACCEPT, value.parse().map_err(|_| ToolError::MissingInput)?);
    }

    if let Some(value) = language {
        headers.insert(ACCEPT_LANGUAGE, value.parse().map_err(|_| ToolError::MissingInput)?);
    }

    let bytes = state.client.get_bytes(url, headers).await?;

    println!("Fetched {} bytes from {}", bytes.len(), url);

    let text = match format {
        Some("application/xml;type=fmx4") | Some("application/fmx4") => parse_fmx4(&bytes)?,
        Some("xhtml") | Some("application/xhtml+xml") => parse_xhtml(&bytes)?,
        Some("pdf") | Some("application/pdf") | Some("pdfa1a") | Some("application/pdf;type=pdfa1a") => parse_pdf(&bytes)?,
        _ => return Err(ToolError::InvalidInput),
    };

    let retrieved_text = search_text(&text, &regex_pattern, max_matches as usize, 10000)?;

    Ok(format!("{:?}", retrieved_text))

}