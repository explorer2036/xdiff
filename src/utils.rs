use std::fmt::{Display, Formatter};

use anyhow::Result;
use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use std::io::Write;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

struct Line(Option<usize>);

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(n) => write!(f, "{:<4}", n + 1),
            None => write!(f, "    "),
        }
    }
}

pub fn build_diff(old: String, new: String) -> Result<String> {
    let diff = TextDiff::from_lines(&old, &new);
    let mut buf = Vec::with_capacity(4096);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(&mut buf, "{:-^1$}", "-", 80)?;
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(
                    &mut buf,
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )?;
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(&mut buf, "{}", s.apply_to(value).underlined().on_black())?;
                    } else {
                        write!(&mut buf, "{}", s.apply_to(value))?;
                    }
                }
                if change.missing_newline() {
                    writeln!(&mut buf)?;
                }
            }
        }
    }

    Ok(String::from_utf8(buf)?)
}

pub fn highlight_text(text: &str, extension: &str) -> Result<String> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension(extension).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    let mut output = String::new();
    for line in LinesWithEndings::from(text) {
        let ranges = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        output.push_str(&escaped);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn diff_text_should_work() {
        let old = "foo\nbar";
        let new = "foo\nbaz";
        let diff = build_diff(old.to_string(), new.to_string()).unwrap();
        let expected = include_str!("../fixtures/diff.txt");
        assert_eq!(diff, expected);
    }

    #[test]
    fn highlight_text_should_work() {
        let source = json!({
            "foo": "bar",
            "baz": "qux"
        });
        let text = serde_json::to_string_pretty(&source).unwrap();
        let expected = include_str!("../fixtures/highlight.txt");
        assert_eq!(highlight_text(&text, "json").unwrap(), expected);
    }
}
