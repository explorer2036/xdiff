use anyhow::{anyhow, Ok, Result};
use clap::{Parser, Subcommand};

/// Diff two http requests and compare the difference of the responses
#[derive(Debug, Parser)]
#[clap(version, author, about, long_about = None)]
pub(crate) struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Action {
    /// Diff two API responses based on given profile
    Run(RunArgs),
}

#[derive(Parser, Debug)]
pub(crate) struct RunArgs {
    /// Profile name
    #[clap(short, long, value_parser)]
    profile: String,

    // Override args
    // For query params, use `-e key=value`
    // For headers, use `-e %key=value`
    // For body, use `-e @key=value`
    #[clap(short, long, value_parser = parse_key_val, number_of_values = 1)]
    extra_args: Vec<KeyVal>,
}

pub(crate) enum KeyValType {
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone)]
pub(crate) struct KeyVal {
    key: String,
    value: String,
}

fn parse_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, '=');

    let retrieve = |v: Option<&str>| -> Result<&str> {
        Ok(v.ok_or(anyhow!("invalid key or value: {}", v.unwrap()))?
            .trim())
    };

    let key = retrieve(parts.next()).unwrap();
    let value = retrieve(parts.next()).unwrap();
    let key_type = match key.chars().next() {
        Some('%') => KeyValType::Header,
        Some('@') => KeyValType::Body,
        _ => KeyValType::Query,
    };

    let key = match key_type {
        KeyValType::Header => key[1..].to_string(),
        KeyValType::Body => key[1..].to_string(),
        _ => key.to_string(),
    };
}
