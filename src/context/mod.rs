mod req;
mod res;

use anyhow::Result;
use http::{header, HeaderMap};
use reqwest::Response;

pub use req::RequestContext;
pub use res::{ResponseContext, ResponseHandler};

fn resolve_content_type(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(';').next())
}

pub fn status_text(res: &Response) -> Result<String> {
    let output = format!("{:?} {}\r\n", res.version(), res.status());
    Ok(output)
}

pub fn headers_text(res: &Response, skip_headers: &[String]) -> Result<String> {
    let mut output = String::new();
    for (k, v) in res.headers().iter() {
        if !skip_headers.iter().any(|s| s == k.as_str()) {
            output.push_str(&format!("{}: {:?}\r\n", k, v));
        }
    }
    Ok(output)
}

pub async fn body_text(res: Response, skip_body: &[String]) -> Result<String> {
    let headers = res.headers().clone();
    let content_type = resolve_content_type(&headers);
    let text = res.text().await?;
    match content_type.as_deref() {
        Some("application/json") => filter_json(&text, &skip_body),
        _ => Ok(text),
    }
}

fn filter_json(text: &str, skip_body: &[String]) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;
    if let serde_json::Value::Object(ref mut obj) = json {
        for key in skip_body {
            obj.remove(key);
        }
    }
    Ok(serde_json::to_string_pretty(&json)?)
}

#[cfg(test)]
mod tests {
    use http::HeaderValue;

    use super::*;

    #[test]
    fn test_resolve_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        assert_eq!(resolve_content_type(&headers), Some("application/json"));
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
        assert_eq!(resolve_content_type(&headers), Some("application/json"));
    }
}
