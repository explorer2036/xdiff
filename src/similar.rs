use std::fmt::{Display, Formatter};

use anyhow::Result;
use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use std::io::Write;

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
