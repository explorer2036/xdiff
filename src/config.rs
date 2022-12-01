use anyhow::{Context, Result};
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
    #[serde(skip_serializing_if = "is_default", default)]
    response: ResponseContext,
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    v == &T::default()
}

impl Config {
    pub fn new(items: HashMap<String, Item>) -> Self {
        Self { items }
    }

    pub async fn load_yaml(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    pub fn from_yaml(content: &str) -> Result<Self> {
        let config = serde_yaml::from_str(content)?;
        Self::validate(&config)?;
        Ok(config)
    }

    pub fn get_item(&self, name: &str) -> Option<&Item> {
        self.items.get(name)
    }

    fn validate(&self) -> Result<()> {
        for (name, item) in self.items.iter() {
            item.validate()
                .context(format!("failed to validate item: {}", name))?;
        }
        Ok(())
    }
}

impl Item {
    pub fn new(
        request1: RequestContext,
        request2: RequestContext,
        response: ResponseContext,
    ) -> Self {
        Self {
            request1,
            request2,
            response,
        }
    }

    pub async fn diff(&self, args: Args) -> Result<String> {
        let response1 = self.request1.send(&args).await?;
        let response2 = self.request2.send(&args).await?;

        let text1 = response1.resolve_text(&self.response).await?;
        let text2 = response2.resolve_text(&self.response).await?;

        build_diff(text1, text2)
    }

    fn validate(&self) -> Result<()> {
        self.request1.validate()?;
        self.request2.validate()?;
        Ok(())
    }
}
