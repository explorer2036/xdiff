pub mod cli;
mod config;
mod request;
mod utils;
use cli::{KeyVal, KeyValType};

pub use config::{Config, Item};
pub use request::{RequestContext, ResponseContext};
pub use utils::{build_diff, highlight_text};

#[derive(Debug, Default, Clone)]
pub struct Args {
    pub query: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
}

impl From<Vec<KeyVal>> for Args {
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
