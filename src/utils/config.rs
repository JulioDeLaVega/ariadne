pub struct Config {
    pub celex_base_url: String,
    pub sparql_endpoint: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            celex_base_url: "https://publications.europa.eu/resource/celex".into(),
            sparql_endpoint: "https://publications.europa.eu/webapi/rdf/sparql".into(),
        }
    }
}