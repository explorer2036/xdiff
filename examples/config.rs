use anyhow::Result;
use xdiff::Config;

fn main() -> Result<()> {
    let content = include_str!("../fixtures/test.yaml");
    let config = Config::from_yaml(content)?;
    println!("{:?}", config);
    Ok(())
}
