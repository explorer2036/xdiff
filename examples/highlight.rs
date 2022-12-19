use std::fs;

use serde_json::json;
use xdiff::highlight_text;

fn main() {
    let source = json!({
        "foo": "bar",
        "baz": "qux"
    });
    let text = serde_json::to_string_pretty(&source).unwrap();
    let content = highlight_text(&text, "json").unwrap();
    fs::write("fixtures/highlight.txt", content).unwrap();
}
