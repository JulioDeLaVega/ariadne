use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum ToolError {
    MissingInput,
    InvalidInput,
    Http(String),
    UnknownTool(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::MissingInput => write!(f, "missing input"),
            ToolError::InvalidInput => write!(f, "invalid input"),
            ToolError::Http(e) => write!(f, "{e}"),
            ToolError::UnknownTool(t) => write!(f, "unknown tool: {t}"),
        }
    }
}

impl From<reqwest::Error> for ToolError {
    fn from(e: reqwest::Error) -> Self {
        ToolError::Http(e.to_string())
    }
}

impl ResponseError for ToolError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ToolError::MissingInput => HttpResponse::BadRequest().body(self.to_string()),
            ToolError::InvalidInput => HttpResponse::BadRequest().body(self.to_string()),
            ToolError::UnknownTool(_) => HttpResponse::NotFound().body(self.to_string()),
            ToolError::Http(_) => HttpResponse::BadGateway().body(self.to_string()),
        }
    }
}