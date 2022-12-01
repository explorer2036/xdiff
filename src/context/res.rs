use anyhow::Result;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use super::{body_text, headers_text, status_text};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ResponseContext {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skip_body: Vec<String>,
}

impl ResponseContext {
    pub fn new(skip_headers: Vec<String>, skip_body: Vec<String>) -> Self {
        Self {
            skip_headers,
            skip_body,
        }
    }
}

#[derive(Debug)]
pub struct ResponseHandler(Response);

impl ResponseHandler {
    pub fn new(res: Response) -> Self {
        Self(res)
    }

    pub fn into_inner(self) -> Response {
        self.0
    }

    pub async fn resolve_text(self, ctx: &ResponseContext) -> Result<String> {
        let res = self.0;

        let mut output = String::new();
        output.push_str(&status_text(&res)?);

        output.push_str(&headers_text(&res, &ctx.skip_headers)?);

        output.push_str(&body_text(res, &ctx.skip_body).await?);
        Ok(output)
    }

    pub fn header_keys(&self) -> Vec<String> {
        self.0
            .headers()
            .iter()
            .map(|(k, _)| k.as_str().to_owned())
            .collect()
    }
}
