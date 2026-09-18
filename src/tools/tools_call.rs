use crate::utils::{eurlex, sparql, Client, Config, ToolError, ted};

pub async fn tools_call(client: &Client, config: &Config, name: &str, input: &str,) -> Result<String, ToolError> {
    match name {
        "eurlex.get_document" => eurlex::get_document(client, config, input).await,
        "cellar.get_works_based_on_keyword" => sparql::run_sparql(client, config, sparql::get_works_based_on_keyword(input)).await,
        "cellar.get_related_acts" => sparql::run_sparql(client, config, sparql::get_related_acts(input)).await,
        "cellar.get_predicates" => sparql::run_sparql(client, config, sparql::get_predicates(input)).await,
        "cellar.uri_from_celex" => sparql::run_sparql(client, config, sparql::uri_from_celex(input)).await,
        "cellar.get_objects_based_on_uri" => {
            sparql::run_sparql(client, config, sparql::get_objects_based_on_uri(input)).await
        }
        "cellar.get_resource_adopts_resource" => {
            sparql::run_sparql(client, config, sparql::get_resource_adopts_resource(input)).await
        }
        "cellar.get_resource_legal_information_miscellaneous" => {
            sparql::run_sparql(client, config, sparql::get_resource_legal_information_miscellaneous(input)).await
        }
        "ted.search_notices" => {
            ted::search_notices(client, input).await
        }
        "ted.search_awards" => {
            ted::search_awards(client, input).await
        }

        other => Err(ToolError::UnknownTool(other.to_string())),
    }
}