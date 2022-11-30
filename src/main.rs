use anyhow::{Ok, Result};
use clap::Parser;
use std::io::stdout;
use std::io::Write;
use xdiff::{
    cli::{Action, Options, RunOptions},
    Config,
};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Options::parse();
    match opts.action {
        Action::Run(args) => run(args).await?,
        _ => panic!("not implemented"),
    }
    Ok(())
}

// cargo run -- run -i rust -a a=100 -a %b=1 -a @c=2
// cargo run -- run -i todo -a a=100 -a %b=1 -a @c=2
async fn run(opts: RunOptions) -> Result<()> {
    let file = opts
        .config
        .unwrap_or_else(|| "fixtures/test.yaml".to_string());
    let config = Config::load_yaml(&file).await?;

    let item = config.get_item(&opts.item).ok_or_else(|| {
        anyhow::anyhow!("profile {} not found in config file {}", opts.item, file)
    })?;
    let args = opts.args.into();
    let output = item.diff(args).await?;
    let mut stdout = stdout().lock();
    write!(stdout, "{}", output)?;

    Ok(())
}
