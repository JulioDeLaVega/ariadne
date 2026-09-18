use crate::utils::{Client, ToolError, Config};

const PREFIX: &str = "PREFIX cdm: <http://publications.europa.eu/ontology/cdm#>";

pub async fn run_sparql(client: &Client, config: &Config, query: String) -> Result<String, ToolError> {
    client.sparql_query(&config.sparql_endpoint, &query).await.map_err(ToolError::from)
}

pub fn get_works_based_on_keyword(input: &str) -> String {
    format!(r#"{PREFIX}
        SELECT DISTINCT ?work ?celex ?title
        WHERE {{
        ?work cdm:work_has_resource-type <http://publications.europa.eu/resource/authority/resource-type/REG> .
        ?work cdm:resource_legal_id_celex ?celex .
        ?expr cdm:expression_belongs_to_work ?work .
        ?expr cdm:expression_title ?title .
        FILTER(CONTAINS(LCASE(STR(?title)), LCASE("{input}")))
        FILTER(lang(?title) = "en" || lang(?title) = "")
        }}
        LIMIT 3"#)
}

pub fn get_related_acts(input: &str) -> String {
    format!(r#"{PREFIX}
        SELECT DISTINCT ?work ?celex ?procedure
        WHERE {{
            ?work cdm:resource_legal_information_miscellaneous ?procedure .
            FILTER(CONTAINS(LCASE(STR(?procedure)), LCASE(STR("{input}"))))
            OPTIONAL {{ ?work cdm:resource_legal_id_celex ?celex . }}
        }}
        LIMIT 30"#)
}

pub fn get_predicates(input: &str) -> String {
    format!(r#"{PREFIX}
        SELECT DISTINCT ?predicate ?object
        WHERE {{
            <{input}> ?predicate ?object .
        }}
        LIMIT 200"#)
}

pub fn uri_from_celex(input: &str) -> String {
    let input = input.to_lowercase();
    format!(r#"{PREFIX}
        SELECT DISTINCT ?work ?celex
        WHERE {{
        ?work cdm:resource_legal_id_celex ?celex .
        FILTER(LCASE(STR(?celex)) = "{input}")
        }}
        LIMIT 3"#)
}

pub fn get_objects_based_on_uri(input: &str) -> String {
    format!(r#"{PREFIX}
        SELECT DISTINCT ?subject ?celex ?type ?resourceType ?title ?date
        WHERE {{
            ?subject cdm:resource_legal_based_on_resource_legal <{input}> .
            OPTIONAL {{ ?subject cdm:resource_legal_id_celex ?celex . }}
            OPTIONAL {{ ?subject cdm:resource_legal_type ?type . }}
            OPTIONAL {{ ?subject cdm:work_has_resource-type ?resourceType . }}
            OPTIONAL {{ ?subject cdm:work_title ?title . }}
            OPTIONAL {{ ?subject cdm:work_date_document ?date . }}
        }}
        LIMIT 200"#)
}

pub fn get_resource_adopts_resource(input: &str) -> String {
    format!(r#"{PREFIX}
        SELECT ?object
        WHERE {{
        <{input}> cdm:resource_legal_adopts_resource_legal ?object .
        }}
        LIMIT 2"#)
}

pub fn get_resource_legal_information_miscellaneous(input: &str) -> String {
    format!(r#"{PREFIX}
        SELECT DISTINCT ?subject ?celex
        WHERE {{
            ?subject cdm:resource_legal_information_miscellaneous ?procedure .
            FILTER(CONTAINS(LCASE(STR(?procedure)), LCASE("{input}")))
            OPTIONAL {{
                ?subject cdm:resource_legal_id_celex ?celex .
            }}
        }}
        LIMIT 30"#)
}