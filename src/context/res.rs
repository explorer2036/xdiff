use anyhow::{Ok, Result};
use reqwest::Response;
use serde::{Deserialize, Serialize};

use super::resolve_content_type;

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

    pub async fn resolve_text(self, ctx: &ResponseContext) -> Result<String> {
        let res = self.0;

        let mut output = String::new();
        output.push_str(&format!("{:?} {}\r\n", res.version(), res.status()).to_owned());

        let headers = res.headers().clone();
        for (k, v) in headers.iter() {
            if !ctx.skip_headers.iter().any(|s| s == k.as_str()) {
                output.push_str(&format!("{}: {:?}\r\n", k, v));
            }
        }

        let text = res.text().await?;
        let content_type = resolve_content_type(&headers);
        match content_type.as_deref() {
            Some("application/json") => {
                let text = filter_json(&text, &ctx.skip_body)?;
                output.push_str(&text);
            }
            _ => {
                output.push_str(&text);
            }
        }
        Ok(output)
    }

    pub fn resolve_header_keys(&self) -> Vec<String> {
        self.0
            .headers()
            .iter()
            .map(|(k, _)| k.as_str().to_owned())
            .collect()
    }
}

fn filter_json(text: &str, skip_body: &[String]) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;
    if let serde_json::Value::Object(ref mut obj) = json {
        for key in skip_body {
            obj.remove(key);
        }
    }
    Ok(serde_json::to_string_pretty(&json)?)
}
