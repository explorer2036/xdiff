use anyhow::{Ok, Result};
use clap::Parser;
use xdiff::{
    cli::{Action, Args, RunArgs},
    Config,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::Run(args) => run(args).await?,
        _ => panic!("not implemented"),
    }
    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_file = args
        .config
        .unwrap_or_else(|| "fixtures/test.yaml".to_string());
    let config = Config::load_yaml(&config_file).await?;
    let profile = config.get_profile(&args.profile).ok_or_else(|| {
        anyhow::anyhow!(
            "profile {} not found in config file {}",
            args.profile,
            config_file
        )
    })?;
    let extra_args = args.extra_args.into();
    profile.diff(extra_args).await?;
    Ok(())
}
