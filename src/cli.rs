use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum KeyValType {
    /// if key has no any prefix, it is for query
    Query,
    /// if key starts with '#', it is for header
    Header,
    /// if key starts with '@', it is for body
    Body,
}

impl Default for KeyValType {
    fn default() -> Self {
        Self::Query
    }
}

#[derive(Debug, Clone)]
pub struct KeyVal {
    pub key_type: KeyValType,
    pub key: String,
    pub value: String,
}

pub fn parse_key_val(s: &str) -> Result<KeyVal> {
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
        key_type,
        key: key.to_string(),
        value: value.to_string(),
    })
}
