use anyhow::Ok;
use http::{HeaderMap, Method, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(flatten)]
    profiles: HashMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    request1: Request,
    request2: Request,
    response: Response,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    #[serde(with = "http_serde::method", default)]
    method: Method,

    #[serde(with = "http_serde::uri", default)]
    url: Uri,

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
pub struct Response {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skip_body: Vec<String>,
}

impl Config {
    pub async fn load_yaml(path: &str) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(content)?)
    }

    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }
}

impl Profile {
    // pub fn diff(&self, args: Args) -> Result<String> {
    //     let response1 = self.request1.send(&args).await?;
    //     let response1 = self.request2.send(&args).await?;
    // }
}
