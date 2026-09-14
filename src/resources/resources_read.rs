use serde_json::{json, Value};

const TED_SEARCH_GUIDE_URI: &str = "ted://search-guide";

pub fn read_resource(uri: &str) -> Result<Value, String> {
    if uri != TED_SEARCH_GUIDE_URI {
        return Err(format!("Resource not found: {}", uri));
    }

    let content = include_str!("../data/ted-search-guide.md");

    Ok(json!({
        "contents": [
            {
                "uri": TED_SEARCH_GUIDE_URI,
                "mimeType": "text/markdown",
                "text": content
            }
        ]
    }))
}