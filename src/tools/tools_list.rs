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
                    "keyword": {
                        "type": "string",
                        "description": "A keyword or phrase to search for within EU regulation titles (e.g. 'data protection'). Matched case-insensitively as a substring."
                    }
                },
                "required": ["keyword"]
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
            description: r"Fetch the text of a cellar document and search it for a pattern, returning matching substrings, each capped in length, up to a maximum number of matches, instead of the full document text. For a cellar legal act, first use cellar.uri_from_celex to find the work URI, then cellar.get_expressions_based_on_work to find the desired language expression, and finally eurlex.get_manifestations_based_on_expression to find the desired manifestation URI. Pass that manifestation URI to this tool. ".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "uri of the resource to fetch. If the uri works but the returned text is empty, you can try to add '/DOC_1' and then '/DOC_2' to the uri, usually the document itself is nested there. Always try first with XHTML, then PDF, then Formex XML (fmx4) if available."
                    },
                    "accept": {
                        "type": "string",
                        "description": "Optional HTTP Accept header specifying the desired response format, for example 'application/xhtml+xml', 'application/pdf;type=pdfa1a', or 'application/xml;type=fmx4'."
                    },
                    "language": {
                        "type": "string",
                        "description": "Optional language for the requested resource using a language code such as 'eng', 'fra', or 'deu'."
                    },
                    "max_matches": {
                        "type": "integer",
                        "description": "Maximum number of matches to return. If you look for something specific, you may want to set this to 5 or 10 so that you can identify where the information is located in the document and then launch a second search with a more specific pattern and a low max_matches (e.g. max_matches = 1) to extract the exact passage. Default is 2."
                    },
                    "regex_pattern": {
                        "type": "string",
                        "description": r"A regular expression (Rust `regex` crate syntax). Include any desired surrounding context directly in the pattern (e.g. `Section 4\\.2.{0,150}` to get the heading plus ~150 trailing characters). No lookaround support. Be careful that the article and the number itself could be separated by html tags or similar dur to formatting, so you may need to include optional whitespace or html tags in the pattern. For example, to match 'Article 12' in a document that may have html tags between the words, use `Article\\s*<[^>]*>\\s*12`. Escape literal regex metacharacters (. ( ) [ ] { } + * ? ^ $ \ |) when searching for text containing them, e.g. 'Article 12(a)' becomes 'Article 12\(a\)'. A malformed pattern returns a compile error describing the issue - revise and retry if necessary."
                    }
                },
                "required": ["url", "regex_pattern"]
            }),
        },
        ToolDefinition {
            name: "edgar.helper.get_cik".into(),
            description: "Resolve a US company ticker symbol to its SEC Central Index Key (CIK) using the SEC EDGAR company ticker database. The ticker lookup is cached in memory after the first request. Returns the CIK as a zero-padded 10-digit string.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "ticker": {
                        "type": "string",
                        "description": "A US company ticker symbol (e.g. 'AAPL', 'MSFT', or 'NVDA')."
                    }
                },
                "required": ["ticker"]
            }),
        },
        ToolDefinition {
            name: "edgar.helper.get_structure".into(),
            description: "Fetch SEC EDGAR Company Facts data for a company identified by its CIK. Returns a catalog of the company's reported financial concepts across SEC taxonomies, including concept tags, labels, units, number of facts, and the earliest and latest reporting periods available for each concept. Use this if you want to discover which data you can fetch for a given company CIK".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "cik": {
                        "type": ["string", "number"],
                        "description": "The company's SEC Central Index Key (CIK). Accepts a numeric CIK or a string, optionally prefixed with 'CIK'. The CIK must contain up to 10 digits; it is automatically zero-padded to 10 digits (e.g. '320193', '0000320193', or 'CIK0000320193' for Apple Inc.)."
                    }
                },
                "required": ["cik"]
            }),
        },
        ToolDefinition {
            name: "edgar.xbrl.get_company_concept".into(),
            description: "Fetch a specific SEC EDGAR XBRL company concept for a company identified by its CIK, taxonomy, and concept tag. Returns the company's reported facts for that concept, including the entity name, concept label and description, units, historical values, reporting periods, and source URL. Use this after discovering an available taxonomy and tag with edgar.get_company_tags when you need the detailed historical facts for a specific concept.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "cik": {
                        "type": ["string", "number"],
                        "description": "The company's SEC Central Index Key (CIK). Accepts a numeric CIK or a string, optionally prefixed with 'CIK'. The CIK must contain up to 10 digits; it is automatically zero-padded to 10 digits (e.g. '320193', '0000320193', or 'CIK0000320193' for Apple Inc.)."
                    },
                    "taxonomy": {
                        "type": "string",
                        "description": "The SEC XBRL taxonomy containing the concept, such as 'us-gaap' or 'dei'."
                    },
                    "tag": {
                        "type": "string",
                        "description": "The XBRL concept tag to retrieve, such as 'Assets', 'RevenueFromContractWithCustomerExcludingAssessedTax', or 'EntityCommonStockSharesOutstanding'."
                    }
                },
                "required": ["cik", "taxonomy", "tag"]
            }),
        },
        ToolDefinition {
            name: "edgar.xbrl.get_frame".into(),
            description: "Fetch an SEC EDGAR XBRL frame containing a standardized financial concept across reporting companies for a specific reporting period. Returns comparable facts from multiple companies, including the taxonomy, concept label and description, unit, reporting frame, entity identifiers, reported values, reporting periods, and source URL. Use this to compare a specific XBRL concept across companies for a common reporting period. Optional filters can narrow results by CIK, accession number, entity name, location, or a numeric value threshold. Multiple filters are combined with AND. Use the company facts tool instead when retrieving the historical facts of one specific company.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "taxonomy": {
                        "type": "string",
                        "description": "The SEC XBRL taxonomy containing the concept, such as 'us-gaap' or 'dei'."
                    },
                    "tag": {
                        "type": "string",
                        "description": "The XBRL concept tag to retrieve, such as 'Revenues', 'Assets', or 'NetIncomeLoss'."
                    },
                    "unit": {
                        "type": "string",
                        "description": "The XBRL unit used for the concept, such as 'USD', 'shares', or 'USD/shares'."
                    },
                    "frame": {
                        "type": "string",
                        "description": "The SEC XBRL reporting frame identifying the reporting period, such as 'CY2025Q4' for calendar-year Q4 2025 or 'CY2025' for calendar-year 2025."
                    },
                    "cik": {
                        "type": "integer",
                        "description": "Optional SEC Central Index Key (CIK). Returns only records for the specified registrant."
                    },
                    "accn": {
                        "type": "string",
                        "description": "Optional SEC accession number. Returns only records with an exact accession number match, such as '0000913760-26-000017'."
                    },
                    "entity_name": {
                        "type": "string",
                        "description": "Optional entity name filter. Matches a case-insensitive substring of the entity name, such as 'NVIDIA' or 'StoneX'."
                    },
                    "loc": {
                        "type": "string",
                        "description": "Optional location filter. Matches the location field exactly, such as 'US-NY'."
                    },
                    "val": {
                        "type": "string",
                        "description": "Optional numeric value threshold. Append '+' to return records whose values are greater than or equal to the threshold, or '-' to return records whose values are less than or equal to the threshold. Examples: '1000000000+' for values of at least one billion, or '1000000-' for values of at most one million. Exact-match filtering is not supported."
                    }
                },
                "required": ["taxonomy", "tag", "unit", "frame"]
            }),
        },
        ToolDefinition {
            name: "edgar.get_submissions".into(),
            description: "Fetch the SEC EDGAR filing history (submissions) for a company identified by its CIK, with optional filters on form type and filing date range. Returns the company's name, tickers, SIC code and fiscal year end, plus a list of matching filings (newest first), each with form type, filing date, report date, accession number, acceptance timestamp, primary document, items (for 8-Ks), and a direct URL to the filing document on sec.gov. Older filings are fetched automatically when the requested date range reaches back beyond the recent-filings window. Use this to find specific filings such as 10-Ks, 10-Qs, 8-Ks, proxy statements or insider forms (3/4/5/144) for a company, e.g. 'the last four 10-Qs' or 'all 8-Ks filed in 2025'.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "cik": {
                        "type": ["string", "number"],
                        "description": "The company's SEC Central Index Key (CIK). Accepts a numeric CIK or a string, optionally prefixed with 'CIK'. The CIK must contain up to 10 digits; it is automatically zero-padded to 10 digits (e.g. '1045810', '0001045810', or 'CIK0001045810' for NVIDIA Corp)."
                    },
                    "form": {
                        "type": ["string", "array"],
                        "items": { "type": "string" },
                        "description": "Optional form type filter, such as '10-Q', '10-K', '8-K', 'DEF 14A', '4' or '144'. Accepts a single form, a comma-separated string (e.g. '10-Q,10-K'), or an array of forms. Matching is case-insensitive and exact, so amendments such as '10-Q/A' must be requested explicitly. If omitted, all form types are returned."
                    },
                    "filing_date_from": {
                        "type": "string",
                        "description": "Optional inclusive start of the filing date range, in YYYY-MM-DD format (e.g. '2025-01-01'). Only filings filed on or after this date are returned."
                    },
                    "filing_date_to": {
                        "type": "string",
                        "description": "Optional inclusive end of the filing date range, in YYYY-MM-DD format (e.g. '2025-12-31'). Only filings filed on or before this date are returned. Must not be earlier than filing_date_from."
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 1000,
                        "default": 100,
                        "description": "Optional maximum number of filings to return, newest first. Defaults to 100. The response includes total_matches so truncation can be detected."
                    }
                },
                "required": ["cik"]
            }),
        },
    ]
}