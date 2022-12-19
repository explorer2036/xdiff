use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

pub mod xdiff;
pub mod xreq;
// pub use xdiff::{DiffConfig, DiffItem};
// pub use xreq::RequestConfig;

#[async_trait]
pub trait Load
where
    Self: Sized + Validate + DeserializeOwned,
{
    /// load config from yaml file
    async fn load_yaml(path: &str) -> Result<Self>
    where
        Self: Sized,
    {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    /// load config from yaml string
    fn from_yaml(content: &str) -> Result<Self>
    where
        Self: Sized,
    {
        let config: Self = serde_yaml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }
}

pub trait Validate {
    fn validate(&self) -> Result<()>;
}
