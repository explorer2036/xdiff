use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{request::ResponseContext, similar::build_diff, Args, RequestContext};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(flatten)]
    items: HashMap<String, Item>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    request1: RequestContext,
    request2: RequestContext,
    response: ResponseContext,
}

impl Config {
    pub async fn load_yaml(path: &str) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(content)?)
    }

    pub fn get_item(&self, name: &str) -> Option<&Item> {
        self.items.get(name)
    }
}

impl Item {
    pub async fn diff(&self, args: Args) -> Result<String> {
        let response1 = self.request1.send(&args).await?;
        let response2 = self.request2.send(&args).await?;

        let text1 = response1.resolve_text(&self.response).await?;
        let text2 = response2.resolve_text(&self.response).await?;

        Ok(build_diff(text1, text2)?)
    }
}
