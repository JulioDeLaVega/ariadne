use serde::{Serialize};
use serde_json::Value;
use serde_json::json;
use serde_json;

#[derive(Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

pub fn tools_list() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "eurlex_url".into(),
            description: "Fetch data from the Eurlex URL endpoint. Only use this tool if you need to retrieve the entire data for a specific document. This may result in too much data. For other tasks, use the SPARQL tools instead.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string", "description": "The document number to send to the Eurlex URL endpoint" }
                },
                "required": ["input"]
            }),
        },
        ToolDefinition {
            name: "sparql_reg_search".into(),
            description: "Search EU regulations by keyword using the EU Publications Office SPARQL endpoint. Given a search term, finds regulations (resource-type REG) whose title contains that term (case-insensitive, English or language-neutral titles only). Returns up to 3 matching results, each with the work URI, CELEX number, and title. Note: input is a plain keyword/phrase, not a full SPARQL query — it gets substituted into a fixed query template.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "A keyword or phrase to search for within EU regulation titles (e.g. 'data protection'). Matched case-insensitively as a substring."
                    }
                },
                "required": ["input"]
            }),
        },
        ToolDefinition {
            name: "sparql_get_predicates".into(),
            description: "Retrieve RDF predicates and objects for a given subject URI using the EU Publications Office SPARQL endpoint.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "The URI of the subject for which to retrieve predicates and objects."
                    }
                },
                "required": ["input"]
            }),
        },
        ToolDefinition {
            name: "sparql_uri_from_celex".into(),
            description: "Retrieve the URI for a given CELEX number using the EU Publications Office SPARQL endpoint.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "The CELEX number for which to retrieve the URI."
                    }
                },
                "required": ["input"]
            }),
        },
        ToolDefinition {
            name: "sparql_get_objects_based_on_uri".into(),
            description: "Retrieve resources (RDF objects) that reference or relate to a given URI using the EU Publications Office SPARQL endpoint. This traverses relationships in the REVERSE direction from sparql_get_predicates: given a work's URI, it finds other works that point AT it — including acts that amend or repeal it, implementing acts, resolutions, communications, working documents, parliament decisions, implementing decisions, Commission reports referencing it, and other documents that cite it. Useful for building an amendment/repeal history (e.g. querying a regulation's URI returns every later act that amends or repeals it), for finding downstream implementing measures, or for surfacing institutional follow-up (resolutions, reports) tied to a piece of legislation. Does not cover national transposition measures for directives, which are not modeled in this graph.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "The URI for which to retrieve resources."
                    }
                },
                "required": ["input"]
            }),
        },
        ToolDefinition {
            name: "sparql_get_resource_adopts_resource".into(),
            description: "Retrieve resources that are adopted by a given resource URI using the EU Publications Office SPARQL endpoint. This tool finds resources that are legally adopted by the specified resource. For instance the initial proposal. By analysing this resource predicates, this can be useful for tracing the legislative process or finding a procedural act and hence identifying related documents.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "The URI for which to retrieve the adopted act legal act."
                    }
                },
                "required": ["input"]
            }),
        },
        ToolDefinition {
            name: "sparql_get_resource_legal_information_miscellaneous".into(),
            description: "Retrieve resources based on miscelaneous information. This tool is mostly useful for finding all procedural acts by querying the tool with the procedure number.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "The legal information miscellaneous value for which to retrieve the resources. If you look for procedural acts, the procedure number."
                    }
                },
                "required": ["input"]
            }),
        },
        ToolDefinition {
            name: "ted_search".into(),
            description: "Search EU public procurement notices using the official TED Search API.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "TED search query using the TED Search API query syntax."
                    }
                },
                "required": ["input"]
            }),
        },
        ToolDefinition {
            name: "ted_award".into(),
            description: "Search EU public procurement awards using the official TED Search API.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "TED search query using the TED Search API query syntax."
                    }
                },
                "required": ["input"]
            }),
        },
    ]
}