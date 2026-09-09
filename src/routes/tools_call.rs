use crate::utils::{eurlex, sparql, Client, Config, ToolError};

pub async fn tools_call(
    client: &Client,
    config: &Config,
    name: &str,
    input: &str,
) -> Result<String, ToolError> {
    match name {
        "eurlex_url" => eurlex::call_eurlex_url(client, config, input).await,
        "sparql_reg_search" => run_sparql(client, config, sparql::reg_search(input)).await,
        "sparql_get_related_acts" => run_sparql(client, config, sparql::get_related_acts(input)).await,
        "sparql_get_predicates" => run_sparql(client, config, sparql::get_predicates(input)).await,
        "sparql_uri_from_celex" => run_sparql(client, config, sparql::uri_from_celex(input)).await,
        "sparql_get_objects_based_on_uri" => {
            run_sparql(client, config, sparql::get_objects_based_on_uri(input)).await
        }
        "sparql_get_resource_adopts_resource" => {
            run_sparql(client, config, sparql::get_resource_adopts_resource(input)).await
        }
        "sparql_get_resource_legal_information_miscellaneous" => {
            run_sparql(client, config, sparql::get_resource_legal_information_miscellaneous(input)).await
        }

        other => Err(ToolError::UnknownTool(other.to_string())),
    }
}

async fn run_sparql(client: &Client, config: &Config, query: String) -> Result<String, ToolError> {
    client.sparql_query(&config.sparql_endpoint, &query).await.map_err(ToolError::from)
}