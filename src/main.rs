use anyhow::Result;
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use dialoguer::MultiSelect;
use std::io::stdout;
use std::io::Write;
use xdiff::Args;
use xdiff::Item;
use xdiff::RequestContext;
use xdiff::ResponseContext;
use xdiff::{
    cli::{Action, Options, RunOptions},
    Config,
};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Options::parse();
    match opts.action {
        Action::Run(args) => run(args).await?,
        Action::Parse => parse().await?,
        _ => panic!("not implemented"),
    }
    Ok(())
}

async fn parse() -> Result<()> {
    let theme = ColorfulTheme::default();

    let url1: String = Input::with_theme(&theme)
        .with_prompt("Enter url1")
        .interact_text()
        .unwrap();
    let request1: RequestContext = url1.parse()?;

    let url2: String = Input::with_theme(&theme)
        .with_prompt("Enter url2")
        .interact_text()
        .unwrap();
    let request2: RequestContext = url2.parse()?;

    let item_name: String = Input::with_theme(&theme)
        .with_prompt("Enter item name")
        .interact_text()
        .unwrap();

    let res = request1.send(&Args::default()).await?;
    let headers = res.resolve_header_keys();
    let chosen = MultiSelect::with_theme(&theme)
        .with_prompt("Select headers to skip")
        .items(&headers)
        .interact()?;
    let skip_headers = chosen
        .iter()
        .map(|i| headers[*i].to_string())
        .collect::<Vec<_>>();

    let response = ResponseContext::new(skip_headers, vec![]);
    let item = Item::new(request1, request2, response);
    let config = Config::new(vec![(item_name, item)].into_iter().collect());
    let output = serde_yaml::to_string(&config)?;
    let mut stdout = stdout().lock();
    write!(stdout, "{}", output)?;
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
