pub struct Config {
    pub sparql_endpoint: String,
    pub ted_endpoint: String,
    pub email: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sparql_endpoint: "https://publications.europa.eu/webapi/rdf/sparql".into(),
            ted_endpoint: "https://api.ted.europa.eu/v3/notices/search".into(),
            email: "contact@example.com".into(),
        }
    }
}