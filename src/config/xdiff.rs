use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Load, Validate};
use crate::{context::ResponseContext, utils::build_diff, Args, RequestContext};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    items: HashMap<String, DiffItem>,
}

impl DiffConfig {
    pub fn new(items: HashMap<String, DiffItem>) -> Self {
        Self { items }
    }

    pub fn get_item(&self, name: &str) -> Option<&DiffItem> {
        self.items.get(name)
    }
}

impl Load for DiffConfig {}

impl Validate for DiffConfig {
    fn validate(&self) -> Result<()> {
        for (name, item) in self.items.iter() {
            item.validate()
                .context(format!("failed to validate item: {}", name))?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffItem {
    req1: RequestContext,
    req2: RequestContext,
    #[serde(skip_serializing_if = "is_default", default)]
    res: ResponseContext,
}

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    v == &T::default()
}

impl DiffItem {
    pub fn new(req1: RequestContext, req2: RequestContext, res: ResponseContext) -> Self {
        Self { req1, req2, res }
    }

    pub async fn diff(&self, args: Args) -> Result<String> {
        let res1 = self.req1.send(&args).await?;
        let res2 = self.req2.send(&args).await?;
        let text1 = res1.resolve_text(&self.res).await?;
        let text2 = res2.resolve_text(&self.res).await?;
        build_diff(text1, text2)
    }

    fn validate(&self) -> Result<()> {
        self.req1.validate()?;
        self.req2.validate()?;
        Ok(())
    }
}
