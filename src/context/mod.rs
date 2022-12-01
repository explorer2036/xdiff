mod req;
mod res;

use http::{header, HeaderMap};

pub use req::RequestContext;
pub use res::{ResponseContext, ResponseHandler};

fn resolve_content_type(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(';').next())
}
