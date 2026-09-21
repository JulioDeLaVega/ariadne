#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Self { inner: reqwest::Client::new() }
    }

    pub async fn get_text(
        &self,
        arguments: &serde_json::Value,
    ) -> Result<String, reqwest::Error> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let accept = arguments
            .get("accept")
            .and_then(|v| v.as_str());

        let language = arguments
            .get("language")
            .and_then(|v| v.as_str());

        let mut request = self.inner.get(url);

        if let Some(accept) = accept {
            request = request.header("Accept", accept);
        }

        if let Some(language) = language {
            request = request.header("Accept-Language", language);
        }

        let text = request
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        Ok(text.chars().take(500).collect())
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