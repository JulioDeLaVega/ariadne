use crate::utils::{ToolError, AppState};
use crate::tools::{cellar, ted, edgar};

pub async fn tools_call(state: &AppState, name: &str, arguments: &serde_json::Value) -> Result<String, ToolError> {
    match name {

        "cellar.get_manifestation_content_with_regex" => cellar::functions::get_manifestation_content_with_regex(state, arguments).await,
        "cellar.get_works_based_on_keyword" => cellar::functions::get_works_based_on_keyword(state, arguments).await,
        "cellar.get_predicates" => cellar::functions::get_predicates(state, arguments).await,
        "cellar.uri_from_celex" => cellar::functions::uri_from_celex(state, arguments).await,
        "cellar.get_objects_based_on_uri" => cellar::functions::get_objects_based_on_uri(state, arguments).await,
        "cellar.get_resource_adopts_resource" => cellar::functions::get_resource_adopts_resource(state, arguments).await,
        "cellar.get_resource_legal_information_miscellaneous" => cellar::functions::get_resource_legal_information_miscellaneous(state, arguments).await,
        "cellar.get_expressions_based_on_work" => cellar::functions::get_expressions_based_on_work(state, arguments).await,
        "cellar.get_manifestations_based_on_expression" => cellar::functions::get_manifestations_based_on_expression(state, arguments).await,
        "ted.search_notices" => ted::functions::search_notices(state, arguments).await,
        "ted.search_awards" => ted::functions::search_awards(state, arguments).await,
        "edgar.get_cik" => edgar::functions::get_cik(state, arguments).await,
        "edgar.get_company_tags" => edgar::functions::get_company_tags(state, arguments).await,
        "edgar.get_company_concept" => edgar::functions::get_company_concept(state, arguments).await,
        other => Err(ToolError::UnknownTool(other.to_string())),
    }
}