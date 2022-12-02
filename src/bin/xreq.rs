use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use std::io::stdout;
use std::io::Write;
use xdiff::body_text;
use xdiff::cli::{parse_key_val, KeyVal};
use xdiff::headers_text;
use xdiff::highlight_text;
use xdiff::status_text;
use xdiff::Load;
use xdiff::RequestConfig;
use xdiff::RequestContext;

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
    #[clap(short, long, default_value = "fixtures/xreq.yaml")]
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

    let url: String = Input::with_theme(&theme)
        .with_prompt("Enter url")
        .interact_text()
        .unwrap();

    let name: String = Input::with_theme(&theme)
        .with_prompt("Enter item name")
        .interact_text()
        .unwrap();

    let request: RequestContext = url.parse()?;
    let config = RequestConfig::new(vec![(name, request)].into_iter().collect());
    let output = serde_yaml::to_string(&config)?;
    let after_highlight = xdiff::highlight_text(&output, "yaml");
    let mut stdout = stdout().lock();
    write!(stdout, "\n{}", after_highlight.unwrap())?;
    Ok(())
}

// cargo run --bin xreq run -i todo
async fn run(opts: RunOptions) -> Result<()> {
    let file = opts
        .config
        .unwrap_or_else(|| "fixtures/xreq.yaml".to_string());
    let config = RequestConfig::load_yaml(&file).await?;

    let item = config.get_item(&opts.item).ok_or_else(|| {
        anyhow::anyhow!("xreq item {} not found in config file {}", opts.item, file)
    })?;
    let args = opts.args.into();
    let url = item.url(&args)?;
    let res = item.send(&args).await?.into_inner();

    let status = status_text(&res)?;
    let headers = headers_text(&res, &[])?;
    let body = body_text(res, &[]).await?;

    let mut output = String::new();
    if atty::is(atty::Stream::Stdout) {
        output.push_str(&format!("url: {}\n\n", url));
        output.push_str(&status);
        output.push_str(&highlight_text(&headers, "yaml")?);
        output.push_str(&highlight_text(&body, "json")?);
    } else {
        output.push_str(&body);
    }

    let mut stdout = std::io::stdout().lock();
    write!(stdout, "{}", output)?;
    Ok(())
}
