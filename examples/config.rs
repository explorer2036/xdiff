use anyhow::Result;
use xdiff::{Load, XDiffConfig};

fn main() -> Result<()> {
    let content = include_str!("../fixtures/test.yaml");
    let config = XDiffConfig::from_yaml(content)?;
    println!("{:?}", config);
    Ok(())
}
