use std::str::FromStr;

use anyhow::{anyhow, Ok, Result};
use http::{
    header::{self, HeaderName},
    HeaderMap, HeaderValue, Method,
};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use crate::Args;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestContext {
    #[serde(with = "http_serde::method", default)]
    method: Method,
    url: Url,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    params: Option<serde_json::Value>,

    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    headers: HeaderMap,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    body: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseContext {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skip_body: Vec<String>,
}

#[derive(Debug)]
pub struct ResponseHandler(Response);

impl ResponseHandler {
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
        let content_type = get_content_type(&headers);
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
}

fn filter_json(text: &str, skip_body: &[String]) -> Result<String> {
    let json: serde_json::Value = serde_json::from_str(text)?;
    match json {
        serde_json::Value::Object(mut obj) => {
            for key in skip_body {
                obj.remove(key);
            }
            Ok(serde_json::to_string_pretty(&obj)?)
        }
        _ => Ok(text.to_string()),
    }
}

fn get_content_type(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().split(';').next())
        .flatten()
}

impl RequestContext {
    pub async fn send(&self, args: &Args) -> Result<ResponseHandler> {
        let (headers, query, body) = self.generate(args)?;
        let client = Client::new();
        let builder = client.request(self.method.clone(), self.url.clone());
        let request = builder.query(&query).headers(headers).body(body).build()?;
        let res = client.execute(request).await?;
        Ok(ResponseHandler(res))
    }

    pub fn generate(&self, args: &Args) -> Result<(HeaderMap, serde_json::Value, String)> {
        let mut headers = self.headers.clone();
        let mut query = self.params.clone().unwrap_or_else(|| json!({}));
        let mut body = self.body.clone().unwrap_or_else(|| json!({}));

        for (k, v) in &args.headers {
            headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
        }
        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }

        for (k, v) in &args.query {
            query[k] = v.parse()?;
        }
        for (k, v) in &args.body {
            body[k] = v.parse()?;
        }

        let content_type = get_content_type(&headers);
        match content_type.as_deref() {
            Some("application/json") => {
                let body = serde_json::to_string(&body)?;
                Ok((headers, query, body))
            }
            _ => Err(anyhow!("unsupported content-type")),
        }
    }
}
