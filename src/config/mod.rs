use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

mod xdiff;
mod xreq;
pub use xdiff::{XDiffConfig, XDiffItem};

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    v == &T::default()
}

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
