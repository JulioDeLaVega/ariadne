use crate::utils::{Client, ToolError, Config};
use reqwest::header::{HeaderMap, ACCEPT, ACCEPT_LANGUAGE};

const PREFIX: &str = "PREFIX cdm: <http://publications.europa.eu/ontology/cdm#>";

pub async fn run_sparql_query(client: &Client, config: &Config, query: String) -> Result<String, ToolError> {
    client.sparql_query(&config.sparql_endpoint, &query).await.map_err(ToolError::from)
}

pub fn get_works_based_on_keyword(arguments: &serde_json::Value) -> String {
    let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("");
    format!(r#"{PREFIX}
        SELECT DISTINCT ?work ?celex ?title
        WHERE {{
        ?work cdm:work_has_resource-type <http://publications.europa.eu/resource/authority/resource-type/REG> .
        ?work cdm:resource_legal_id_celex ?celex .
        ?expr cdm:expression_belongs_to_work ?work .
        ?expr cdm:expression_title ?title .
        FILTER(CONTAINS(LCASE(STR(?title)), LCASE("{text}")))
        FILTER(lang(?title) = "en" || lang(?title) = "")
        }}
        LIMIT 3"#)
}

// pub fn get_related_acts(input: &str) -> String {
//     format!(r#"{PREFIX}
//         SELECT DISTINCT ?work ?celex ?procedure
//         WHERE {{
//             ?work cdm:resource_legal_information_miscellaneous ?procedure .
//             FILTER(CONTAINS(LCASE(STR(?procedure)), LCASE(STR("{input}"))))
//             OPTIONAL {{ ?work cdm:resource_legal_id_celex ?celex . }}
//         }}
//         LIMIT 30"#)
// }

pub fn get_predicates(arguments: &serde_json::Value) -> String {
    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");
    format!(r#"{PREFIX}
        SELECT DISTINCT ?predicate ?object
        WHERE {{
            <{uri}> ?predicate ?object .
        }}
        LIMIT 200"#)
}

pub fn uri_from_celex(arguments: &serde_json::Value) -> String {

    let celex = arguments.get("celex").and_then(|v| v.as_str()).unwrap_or("");
    let input = celex.to_lowercase();

    format!(r#"{PREFIX}
        SELECT DISTINCT ?work ?celex
        WHERE {{
        ?work cdm:resource_legal_id_celex ?celex .
        FILTER(LCASE(STR(?celex)) = "{input}")
        }}
        LIMIT 3"#)

}

pub fn get_objects_based_on_uri(arguments: &serde_json::Value) -> String {
    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");
    format!(r#"{PREFIX}
        SELECT DISTINCT ?subject ?celex ?type ?resourceType ?title ?date
        WHERE {{
            ?subject cdm:resource_legal_based_on_resource_legal <{uri}> .
            OPTIONAL {{ ?subject cdm:resource_legal_id_celex ?celex . }}
            OPTIONAL {{ ?subject cdm:resource_legal_type ?type . }}
            OPTIONAL {{ ?subject cdm:work_has_resource-type ?resourceType . }}
            OPTIONAL {{ ?subject cdm:work_title ?title . }}
            OPTIONAL {{ ?subject cdm:work_date_document ?date . }}
        }}
        LIMIT 200"#)
}

pub fn get_resource_adopts_resource(arguments: &serde_json::Value) -> String {
    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");
    format!(r#"{PREFIX}
        SELECT ?object
        WHERE {{
        <{uri}> cdm:resource_legal_adopts_resource_legal ?object .
        }}
        LIMIT 2"#)
}

pub fn get_resource_legal_information_miscellaneous(arguments: &serde_json::Value) -> String {
    
    let text = arguments.get("text").and_then(|v| v.as_str()).unwrap_or("");

    format!(r#"{PREFIX}
        SELECT DISTINCT ?subject ?celex
        WHERE {{
            ?subject cdm:resource_legal_information_miscellaneous ?procedure .
            FILTER(CONTAINS(LCASE(STR(?procedure)), LCASE("{text}")))
            OPTIONAL {{
                ?subject cdm:resource_legal_id_celex ?celex .
            }}
        }}
        LIMIT 30"#)

}

pub fn get_expressions_based_on_work(arguments: &serde_json::Value) -> String {

    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");

    format!(r#"{PREFIX}
        SELECT DISTINCT ?expression ?language ?title
        WHERE {{
            ?expression cdm:expression_belongs_to_work <{uri}> .
            OPTIONAL {{ ?expression cdm:expression_uses_language ?language . }}
            OPTIONAL {{ ?expression cdm:expression_title ?title . }}
        }}
        LIMIT 100"#)

}

pub fn get_manifestations_based_on_expression(arguments: &serde_json::Value) -> String {

    let uri = arguments.get("uri").and_then(|v| v.as_str()).unwrap_or("");

    format!(r#"{PREFIX}
        SELECT DISTINCT ?manifestation ?type ?item
        WHERE {{
            ?manifestation cdm:manifestation_manifests_expression <{uri}> .
            OPTIONAL {{ ?manifestation cdm:manifestation_type ?type . }}
            OPTIONAL {{ ?manifestation cdm:manifestation_has_item ?item . }}
        }}
        LIMIT 100"#)

}

pub async fn get_manifestation_content(
    client: &Client,
    arguments: &serde_json::Value,
) -> Result<String, ToolError> {
    let url = arguments
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or(ToolError::MissingInput)?;

    let mut headers = HeaderMap::new();

    if let Some(value) = arguments.get("accept").and_then(|v| v.as_str()) {
        headers.insert(
            ACCEPT,
            value.parse().map_err(|_| ToolError::MissingInput)?,
        );
    }

    if let Some(value) = arguments.get("language").and_then(|v| v.as_str()) {
        headers.insert(
            ACCEPT_LANGUAGE,
            value.parse().map_err(|_| ToolError::MissingInput)?,
        );
    }

    Ok(client.get_text(url, headers).await?)
}