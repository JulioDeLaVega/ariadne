pub mod client;
pub mod config;
pub mod error;
pub mod eurlex;
pub mod sparql;
pub mod ted;

pub use client::Client;
pub use config::Config;
pub use error::ToolError;
pub use eurlex::call_eurlex_url;