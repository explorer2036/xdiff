use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{is_default, Load, Validate};
use crate::{context::ResponseContext, utils::build_diff, Args, RequestContext};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XDiffConfig {
    #[serde(flatten)]
    items: HashMap<String, XDiffItem>,
}

impl XDiffConfig {
    pub fn new(items: HashMap<String, XDiffItem>) -> Self {
        Self { items }
    }

    pub fn get_item(&self, name: &str) -> Option<&XDiffItem> {
        self.items.get(name)
    }
}

impl Load for XDiffConfig {}

impl Validate for XDiffConfig {
    fn validate(&self) -> Result<()> {
        for (name, item) in self.items.iter() {
            item.validate()
                .context(format!("failed to validate item: {}", name))?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XDiffItem {
    req1: RequestContext,
    req2: RequestContext,
    #[serde(skip_serializing_if = "is_default", default)]
    res: ResponseContext,
}

impl XDiffItem {
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
