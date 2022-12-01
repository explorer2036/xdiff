use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use dialoguer::MultiSelect;
use std::io::stdout;
use std::io::Write;
use xdiff::cli::{parse_key_val, KeyVal};
use xdiff::Args;
use xdiff::Load;
use xdiff::RequestContext;
use xdiff::ResponseContext;
use xdiff::XDiffConfig;
use xdiff::XDiffItem;

/// Diff two http requests and compare the difference of the responses
#[derive(Debug, Parser)]
#[clap(version, author, about, long_about = None)]
pub struct Options {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum Action {
    /// Diff two API responses based on given profile
    Run(RunOptions),
    /// Parse URLs to generate a profile
    Parse,
}

#[derive(Parser, Debug)]
pub struct RunOptions {
    /// Item name
    #[clap(short, long, value_parser)]
    pub item: String,

    /// They are used to override the query, headers and body of the request.
    /// For query params, use `-e key=value`
    /// For headers, use `-e %key=value`
    /// For body, use `-e @key=value`
    #[clap(short, long, value_parser = parse_key_val, number_of_values = 1)]
    pub args: Vec<KeyVal>,

    /// Configuration to use for diff
    #[clap(short, long, default_value = "fixtures/test.yaml")]
    pub config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Options::parse();
    match opts.action {
        Action::Run(args) => run(args).await?,
        Action::Parse => parse().await?,
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
    let item = XDiffItem::new(request1, request2, response);
    let config = XDiffConfig::new(vec![(item_name, item)].into_iter().collect());
    let output = serde_yaml::to_string(&config)?;
    let after_highlight = xdiff::highlight_text(&output, "yaml");
    let mut stdout = stdout().lock();
    write!(stdout, "\n{}", after_highlight.unwrap())?;
    Ok(())
}

// cargo run --bin xdiff run -i todo -a a=100 -a %b=1 -a @c=2
// cargo run --bin xdiff run -i rust -a a=100 -a %b=1 -a @c=2
async fn run(opts: RunOptions) -> Result<()> {
    let file = opts
        .config
        .unwrap_or_else(|| "fixtures/test.yaml".to_string());
    let config = XDiffConfig::load_yaml(&file).await?;

    let item = config.get_item(&opts.item).ok_or_else(|| {
        anyhow::anyhow!("profile {} not found in config file {}", opts.item, file)
    })?;
    let args = opts.args.into();
    let output = item.diff(args).await?;
    let mut stdout = stdout().lock();
    write!(stdout, "{}", output)?;

    Ok(())
}
