use crate::utils::client::Client;
use crate::utils::{config::Config, error::ToolError};

pub async fn call_eurlex_url(client: &Client, config: &Config, input: &str) -> Result<String, ToolError> {
    let url = format!("{}/{}?language=eng", config.celex_base_url, input);
    client.get_text(&url).await.map_err(ToolError::from)
}