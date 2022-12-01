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

    #[serde(skip_serializing_if = "empty_json_value", default)]
    params: Option<serde_json::Value>,

    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    headers: HeaderMap,

    #[serde(skip_serializing_if = "empty_json_value", default)]
    body: Option<serde_json::Value>,
}

fn empty_json_value(v: &Option<serde_json::Value>) -> bool {
    v.as_ref()
        .map_or(true, |v| v.as_object().unwrap().is_empty())
}

impl RequestContext {
    fn new(
        method: Method,
        url: Url,
        params: Option<serde_json::Value>,
        headers: HeaderMap,
        body: Option<serde_json::Value>,
    ) -> Self {
        Self {
            method,
            url,
            params,
            headers,
            body,
        }
    }
}

impl FromStr for RequestContext {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut url = Url::parse(s)?;
        let pairs = url.query_pairs();
        let mut params = json!({});
        for (key, value) in pairs {
            params[&*key] = value.parse()?;
        }
        url.set_query(None);

        Ok(RequestContext::new(
            Method::GET,
            url,
            Some(params),
            HeaderMap::new(),
            None,
        ))
    }
}

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

fn resolve_content_type(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(';').next())
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

    pub fn validate(&self) -> Result<()> {
        if let Some(params) = self.params.as_ref() {
            if !params.is_object() {
                return Err(anyhow!(
                    "loading config: params must be an object\n{}",
                    serde_yaml::to_string(params)?
                ));
            }
        }
        if let Some(body) = self.body.as_ref() {
            if !body.is_object() {
                return Err(anyhow!(
                    "loading config: body must be an object\n{}",
                    serde_yaml::to_string(body)?
                ));
            }
        }
        Ok(())
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

        let content_type = resolve_content_type(&headers);
        match content_type.as_deref() {
            Some("application/json") => {
                let body = serde_json::to_string(&body)?;
                Ok((headers, query, body))
            }
            _ => Err(anyhow!("unsupported content-type")),
        }
    }
}
