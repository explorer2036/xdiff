use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

/// Diff two http requests and compare the difference of the responses
#[derive(Debug, Parser)]
#[clap(version, author, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
#[non_exhaustive]
pub enum Action {
    /// Diff two API responses based on given profile
    Run(RunArgs),
}

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Profile name
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// Override args, which could be used to override the query, headers and body of the request.
    /// For query params, use `-e key=value`
    /// For headers, use `-e %key=value`
    /// For body, use `-e @key=value`
    #[clap(short, long, value_parser = parse_key_val, number_of_values = 1)]
    pub extra_args: Vec<KeyVal>,

    /// Configuration to use for diff
    #[clap(short, long, default_value = "fixtures/test.yaml")]
    pub config: Option<String>,
}

#[derive(Debug, Clone)]
pub enum KeyValType {
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone)]
pub struct KeyVal {
    pub key_type: KeyValType,
    pub key: String,
    pub value: String,
}

fn parse_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, '=');

    let key = parts.next().ok_or_else(|| anyhow!("invalid key"))?;
    let value = parts.next().ok_or_else(|| anyhow!("invalid value"))?;
    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Header, key[1..].to_string()),
        Some('@') => (KeyValType::Body, key[1..].to_string()),
        Some(v) if v.is_ascii_alphabetic() => (KeyValType::Query, key.to_string()),
        _ => Err(anyhow!("invalid key: {}", key))?,
    };

    Ok(KeyVal {
        key_type: key_type,
        key: key.to_string(),
        value: value.to_string(),
    })
}
