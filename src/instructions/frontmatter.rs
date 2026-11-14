//! Read `name:` and `description:` out of a file's frontmatter. Pure: text in, fields out.

use std::collections::HashMap;

/// The fields of a file's frontmatter block, or `None` when the file has no block.
///
/// The block opens with `---` on the first line and closes at the next line that starts with
/// `---`. Only single-line `key: value` scalars are read; a folded YAML value would need a real
/// parser, which `.agents/rules/repository.md` rules out as a dependency added to serve one call
/// site. A field this misses shows up as a missing description, which startup refuses, rather
/// than as an empty one served silently.
pub fn parse(text: &str) -> Option<HashMap<String, String>> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let end = rest.find("\n---")?;
    let block = rest[..end].strip_suffix('\r').unwrap_or(&rest[..end]);

    let fields = block
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter_map(field)
        .collect();
    Some(fields)
}

/// One `key: value` line, or `None` when the line is not one.
///
/// The key is `[A-Za-z_][A-Za-z0-9_]*`. The value is everything after the colon and the spaces
/// or tabs that follow it, trimmed; it may be empty.
fn field(line: &str) -> Option<(String, String)> {
    let (key, value) = line.split_once(':')?;
    let mut chars = key.chars();
    let key_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !key_ok {
        return None;
    }
    let value = value.trim_start_matches([' ', '\t']);
    if value.starts_with(char::is_whitespace) {
        return None;
    }
    Some((key.to_owned(), value.trim().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_single_line_fields() {
        let fields = parse("---\nname: x\ndescription:  Says what.  \n---\nbody\n").unwrap();
        assert_eq!(fields["name"], "x");
        assert_eq!(fields["description"], "Says what.");
    }

    #[test]
    fn crlf_is_accepted() {
        let fields = parse("---\r\nname: x\r\ndescription: d\r\n---\r\nbody").unwrap();
        assert_eq!(fields["description"], "d");
    }

    #[test]
    fn no_block_is_none() {
        assert!(parse("# Title\n").is_none());
        assert!(parse("---\nname: x\n").is_none());
    }

    #[test]
    fn folded_values_and_bad_keys_are_skipped() {
        let fields = parse("---\ndescription: >\n  folded\n1bad: v\nempty:\n---\n").unwrap();
        assert_eq!(fields["description"], ">");
        assert!(!fields.contains_key("1bad"));
        assert_eq!(fields["empty"], "");
    }

    #[test]
    fn a_colon_in_the_value_is_kept() {
        let fields = parse("---\ndescription: a: b\n---\n").unwrap();
        assert_eq!(fields["description"], "a: b");
    }
}
