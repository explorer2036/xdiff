use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Load, Validate};
use crate::RequestContext;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestConfig {
    #[serde(flatten)]
    items: HashMap<String, RequestContext>,
}

impl Load for RequestConfig {}

impl RequestConfig {
    pub fn new(items: HashMap<String, RequestContext>) -> Self {
        Self { items }
    }

    pub fn get_item(&self, name: &str) -> Option<&RequestContext> {
        self.items.get(name)
    }
}

impl Validate for RequestConfig {
    fn validate(&self) -> Result<()> {
        for (name, item) in self.items.iter() {
            item.validate()
                .context(format!("failed to validate item: {}", name))?;
        }
        Ok(())
    }
}
