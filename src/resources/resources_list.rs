const TED_SEARCH_GUIDE_URI: &str = "ted://search-guide";

use serde::{Serialize};

#[derive(Serialize)]
pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub mimetype: String,
}

pub fn list_resources() -> Vec<ResourceDefinition> {
    vec![ResourceDefinition {
        uri: TED_SEARCH_GUIDE_URI.to_string(),
        name: "ted-search-guide".to_string(),
        title: "TED Search API Guide".to_string(),
        description: "Guide for using the TED Search API (for the ted.search_notice and ted.search_award mcp tools). Use this resource when constructing TED Expert Search queries.".to_string(),
        mimetype: "text/markdown".to_string(),
    }]
}