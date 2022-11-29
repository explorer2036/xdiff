pub mod cli;
mod config;
use cli::{KeyVal, KeyValType};

pub use cli::Args;
pub use config::{Config, Profile, Request, Response};

#[derive(Debug, Clone)]
pub struct ExtraArgs {
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(args: Vec<KeyVal>) -> Self {
        let mut query = vec![];
        let mut headers = vec![];
        let mut body = vec![];

        for arg in args {
            match arg.key_type {
                KeyValType::Query => query.push((arg.key, arg.value)),
                KeyValType::Header => headers.push((arg.key, arg.value)),
                KeyValType::Body => body.push((arg.key, arg.value)),
            }
        }

        Self {
            query,
            headers,
            body,
        }
    }
}
