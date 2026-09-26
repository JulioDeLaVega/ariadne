use crate::utils::{cellar, Client, Config, ToolError, ted};

pub async fn tools_call(client: &Client, config: &Config, name: &str, arguments: &serde_json::Value) -> Result<String, ToolError> {
    match name {
        "cellar.get_manifestation_content_with_regex" => cellar::get_manifestation_content_with_regex(client, arguments).await,
        "cellar.get_works_based_on_keyword" => cellar::run_sparql_query(client, config, cellar::get_works_based_on_keyword(arguments)).await,
        "cellar.get_predicates" => cellar::run_sparql_query(client, config, cellar::get_predicates(arguments)).await,
        "cellar.uri_from_celex" => cellar::run_sparql_query(client, config, cellar::uri_from_celex(arguments)).await,
        "cellar.get_objects_based_on_uri" => {
            cellar::run_sparql_query(client, config, cellar::get_objects_based_on_uri(arguments)).await
        }
        "cellar.get_resource_adopts_resource" => {
            cellar::run_sparql_query(client, config, cellar::get_resource_adopts_resource(arguments)).await
        }
        "cellar.get_resource_legal_information_miscellaneous" => {
            cellar::run_sparql_query(client, config, cellar::get_resource_legal_information_miscellaneous(arguments)).await
        }
        "cellar.get_expressions_based_on_work" => {
            cellar::run_sparql_query(client, config, cellar::get_expressions_based_on_work(arguments)).await
        }
        "cellar.get_manifestations_based_on_expression" => {
            cellar::run_sparql_query(client, config, cellar::get_manifestations_based_on_expression(arguments)).await
        }
        "ted.search_notices" => {
            ted::search_notices(client, arguments).await
        }
        "ted.search_awards" => {
            ted::search_awards(client, arguments).await
        }

        other => Err(ToolError::UnknownTool(other.to_string())),
    }
}