pub mod client;
pub mod config;
pub mod error;
pub mod eurlex;
pub mod sparql;
pub mod ted;

pub use client::Client;
pub use config::Config;
pub use error::ToolError;
pub use eurlex::get_document;
pub use sparql::run_sparql;
pub use ted::search_notices;
pub use ted::search_awards;