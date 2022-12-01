use anyhow::Result;
use xdiff::{DiffConfig, Load};

fn main() -> Result<()> {
    let content = include_str!("../fixtures/test.yaml");
    let config = DiffConfig::from_yaml(content)?;
    println!("{:?}", config);
    Ok(())
}
