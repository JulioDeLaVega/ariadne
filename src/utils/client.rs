#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Self { inner: reqwest::Client::new() }
    }

    pub async fn get_text(&self, url: &str) -> Result<String, reqwest::Error> {
        self.inner.get(url).send().await?.text().await
    }

    pub async fn sparql_query(&self, endpoint: &str, query: &str) -> Result<String, reqwest::Error> {
        self.inner
            .get(endpoint)
            .query(&[("query", query), ("format", "application/sparql-results+json")])
            .send()
            .await?
            .text()
            .await
    }

    pub async fn post_json(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<String, reqwest::Error> {
        self.inner
            .post(url)
            .json(body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    }
    
}