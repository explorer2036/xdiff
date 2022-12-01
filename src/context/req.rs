use std::str::FromStr;

use anyhow::{anyhow, Ok, Result};
use http::{
    header::{self, HeaderName},
    HeaderMap, HeaderValue, Method,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use crate::Args;

use super::{resolve_content_type, ResponseHandler};

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

impl RequestContext {
    pub async fn send(&self, args: &Args) -> Result<ResponseHandler> {
        let (headers, query, body) = self.generate(args)?;
        let client = Client::new();
        let builder = client.request(self.method.clone(), self.url.clone());
        let request = builder.query(&query).headers(headers).body(body).build()?;
        let res = client.execute(request).await?;
        Ok(ResponseHandler::new(res))
    }

    pub fn url(&self, args: &Args) -> Result<String> {
        let mut url = self.url.clone();
        let mut query = self.params.clone().unwrap_or_else(|| json!({}));
        for (k, v) in &args.query {
            query[k] = v.parse()?;
        }
        if !query.as_object().unwrap().is_empty() {
            let query = serde_qs::to_string(&query)?;
            url.set_query(Some(&query));
        }
        Ok(url.to_string())
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
