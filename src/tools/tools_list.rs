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
            name: "cellar.get_works_based_on_keyword".into(),
            description: "Search EU regulations by keyword using the EU Publications Office SPARQL endpoint. Given a search term, finds regulations (resource-type REG) whose title contains that term (case-insensitive, English or language-neutral titles only). Returns up to 3 matching results, each with the work URI, CELEX number, and title. Note: input is a plain keyword/phrase, not a full SPARQL query — it gets substituted into a fixed query template.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "A keyword or phrase to search for within EU regulation titles (e.g. 'data protection'). Matched case-insensitively as a substring."
                    }
                },
                "required": ["text"]
            }),
        },
        ToolDefinition {
            name: "cellar.get_predicates".into(),
            description: "Retrieve RDF predicates and objects for a given subject URI using the EU Publications Office SPARQL endpoint.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "uri": {
                        "type": "string",
                        "description": "The URI of the subject for which to retrieve predicates and objects."
                    }
                },
                "required": ["uri"]
            }),
        },
        ToolDefinition {
            name: "cellar.uri_from_celex".into(),
            description: "Retrieve the URI for a given CELEX number using the EU Publications Office SPARQL endpoint.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "celex": {
                        "type": "string",
                        "description": "The CELEX number for which to retrieve the URI."
                    }
                },
                "required": ["celex"]
            }),
        },
        ToolDefinition {
            name: "cellar.get_objects_based_on_uri".into(),
            description: "Retrieve resources (RDF objects) that reference or relate to a given URI using the EU Publications Office SPARQL endpoint. This traverses relationships in the REVERSE direction from sparql_get_predicates: given a work's URI, it finds other works that point AT it — including acts that amend or repeal it, implementing acts, resolutions, communications, working documents, parliament decisions, implementing decisions, Commission reports referencing it, and other documents that cite it. Useful for building an amendment/repeal history (e.g. querying a regulation's URI returns every later act that amends or repeals it), for finding downstream implementing measures, or for surfacing institutional follow-up (resolutions, reports) tied to a piece of legislation. Does not cover national transposition measures for directives, which are not modeled in this graph.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "uri": {
                        "type": "string",
                        "description": "The URI for which to retrieve resources."
                    }
                },
                "required": ["uri"]
            }),
        },
        ToolDefinition {
            name: "cellar.get_resource_adopts_resource".into(),
            description: "Retrieve resources that are adopted by a given resource URI using the EU Publications Office SPARQL endpoint. This tool finds resources that are legally adopted by the specified resource. For instance the initial proposal. By analysing this resource predicates, this can be useful for tracing the legislative process or finding a procedural act and hence identifying related documents.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "uri": {
                        "type": "string",
                        "description": "The URI for which to retrieve the adopted act legal act."
                    }
                },
                "required": ["uri"]
            }),
        },
        ToolDefinition {
            name: "cellar.get_resource_legal_information_miscellaneous".into(),
            description: "Retrieve resources based on miscelaneous information. This tool is mostly useful for finding all procedural acts by querying the tool with the procedure number.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The legal information miscellaneous value for which to retrieve the resources. If you look for procedural acts, the procedure number."
                    }
                },
                "required": ["text"]
            }),
        },
        ToolDefinition {
            name: "cellar.get_expressions_based_on_work".into(),
            description: "Retrieve the expressions (language versions) of a work using the EU Publications Office SPARQL endpoint. Given a work's URI (for example obtained via cellar.uri_from_celex), it returns one row per expression with its expression URI, language and title. In the CELLAR data model links point from child to parent (expression_belongs_to_work), so a work's own predicates do not list its expressions and cellar.get_predicates on a work will not reveal them: use this tool instead. Use it as the first step to discover which language versions exist, then pass an expression URI to cellar.get_manifestations_based_on_expression to see the available formats. Language and title may be missing for some expressions.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "uri": {
                        "type": "string",
                        "description": "The URI of the work for which to retrieve expressions, e.g. http://publications.europa.eu/resource/cellar/3e485e15-11bd-11e6-ba9a-01aa75ed71a1."
                    }
                },
                "required": ["uri"]
            }),
        },
        ToolDefinition {
            name: "cellar.get_manifestations_based_on_expression".into(),
            description: "Retrieve the manifestations (concrete formats) of an expression using the EU Publications Office SPARQL endpoint. Given an expression's URI (a single language version of a work, for example obtained via cellar.get_expressions_based_on_work), it returns the manifestation URI, its manifestation_type (for example pdfa1a, fmx4 for Formex 4 XML, or xhtml) and the item URIs holding the actual files. A manifestation can have several items, so a manifestation may appear on multiple rows. In the CELLAR data model links point from child to parent (manifestation_manifests_expression), so an expression's own predicates do not list its manifestations: use this tool instead. Use it to check which formats are available for a given document and language before downloading.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "uri": {
                        "type": "string",
                        "description": "The URI of the expression for which to retrieve manifestations, e.g. http://publications.europa.eu/resource/cellar/3e485e15-11bd-11e6-ba9a-01aa75ed71a1.0006 (the English version of the GDPR)."
                    }
                },
                "required": ["uri"]
            }),
        },
        ToolDefinition {
            name: "ted.search_notices".into(),
            description: "Search EU public procurement notices using the official TED Search API.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "TED search query using the TED Search API query syntax. Use the language of the target country in the query for best results. Use the resource ted-search-guide to learn how to construct queries for the TED Search API."
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "ted.search_awards".into(),
            description: "Search EU public procurement awards using the official TED Search API.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "TED search query using the TED Search API query syntax. Use the language of the target country in the query for best results. Use the resource ted-search-guide to learn how to construct queries for the TED Search API."
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "cellar.get_manifestation_content_with_regex".into(),
            description: r"Fetch the text of a Eur-Lex document or other text resource and search it for a pattern, returning matching substrings, each capped in length, up to a maximum number of matches, instead of the full document text. For a Eur-Lex legal act, first use eurlex.uri_from_celex to find the work URI, then eurlex.get_expressions_based_on_work to find the desired language expression, and finally eurlex.get_manifestations_based_on_expression to find the desired manifestation URI. Pass that manifestation URI to this tool. The optional Accept and language parameters can be used to request a specific representation and language. The 'pattern' parameter is a regular expression (Rust 'regex' crate syntax, linear-time, no lookahead/lookbehind) used to locate a specific passage - an article, section, clause, or defined term - within the fetched text. There is no separate context parameter: to capture text around a match, include it directly in the pattern, e.g. 'Article \d+.{0,200}' to get 'Article 12' plus up to 200 trailing characters. '.' matches any character including newlines, so bound quantifiers ('{0,200}', not '*' or '+') to keep results predictable and avoid crossing into unrelated sections. Escape literal regex metacharacters (. ( ) [ ] { } + * ? ^ $ \ |) when searching for text containing them, e.g. 'Article 12(a)' becomes 'Article 12\(a\)'. A malformed pattern returns a compile error describing the issue - revise and retry if necessary.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL of the resource to fetch. For Eur-Lex documents, this should normally be a manifestation URI obtained through eurlex.uri_from_celex → eurlex.get_expressions_based_on_work → eurlex.get_manifestations_based_on_expression."
                    },
                    "accept": {
                        "type": "string",
                        "description": "Optional HTTP Accept header specifying the desired response format, for example 'application/xhtml+xml', 'application/pdf;type=pdfa1a', or 'application/xml;type=fmx4'."
                    },
                    "language": {
                        "type": "string",
                        "description": "Optional language for the requested resource using a language code such as 'eng', 'fra', or 'deu'."
                    },
                    "regex_pattern": {
                        "type": "string",
                        "description": "A regular expression (Rust `regex` crate syntax). Include any desired surrounding context directly in the pattern (e.g. `Section 4\\.2.{0,150}` to get the heading plus ~150 trailing characters). No lookaround support. Be careful that the article and the number itself could be separated by html tags or similar dur to formatting, so you may need to include optional whitespace or html tags in the pattern. For example, to match 'Article 12' in a document that may have html tags between the words, use `Article\\s*<[^>]*>\\s*12`."
                    }
                },
                "required": ["url", "regex_pattern"]
            }),
        },
    ]
}