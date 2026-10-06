use crate::utils::{Client, Config, ToolError};
use crate::tools::{cellar, ted, edgar};

pub async fn tools_call(client: &Client, config: &Config, name: &str, arguments: &serde_json::Value) -> Result<String, ToolError> {
    match name {
        "cellar.get_manifestation_content_with_regex" => cellar::functions::get_manifestation_content_with_regex(client, arguments).await,
        "cellar.get_works_based_on_keyword" => cellar::functions::run_sparql_query(client, config, cellar::functions::get_works_based_on_keyword(arguments)).await,
        "cellar.get_predicates" => cellar::functions::run_sparql_query(client, config, cellar::functions::get_predicates(arguments)).await,
        "cellar.uri_from_celex" => cellar::functions::run_sparql_query(client, config, cellar::functions::uri_from_celex(arguments)).await,
        "cellar.get_objects_based_on_uri" => {
            cellar::functions::run_sparql_query(client, config, cellar::functions::get_objects_based_on_uri(arguments)).await
        }
        "cellar.get_resource_adopts_resource" => {
            cellar::functions::run_sparql_query(client, config, cellar::functions::get_resource_adopts_resource(arguments)).await
        }
        "cellar.get_resource_legal_information_miscellaneous" => {
            cellar::functions::run_sparql_query(client, config, cellar::functions::get_resource_legal_information_miscellaneous(arguments)).await
        }
        "cellar.get_expressions_based_on_work" => {
            cellar::functions::run_sparql_query(client, config, cellar::functions::get_expressions_based_on_work(arguments)).await
        }
        "cellar.get_manifestations_based_on_expression" => {
            cellar::functions::run_sparql_query(client, config, cellar::functions::get_manifestations_based_on_expression(arguments)).await
        }
        "ted.search_notices" => {
            ted::functions::search_notices(client, arguments).await
        }
        "ted.search_awards" => {
            ted::functions::search_awards(client, arguments).await
        }
        "edgar.get_cik" => {
            edgar::functions::get_cik(client, arguments).await
        }
        "edgar.get_company_concepts" => {
            edgar::functions::get_company_concepts(client, arguments).await
        }

        other => Err(ToolError::UnknownTool(other.to_string())),
    }
}