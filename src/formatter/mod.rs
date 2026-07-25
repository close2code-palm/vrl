//! Canonical source formatting for VRL.
//!
//! Formatting validates input through the parser before applying source-aware
//! whitespace rules. Tokenization preserves comments and literals intact, and
//! the printer applies the canonical layout rules.

mod printer;
mod tokenizer;

use crate::parser;
use printer::Formatter;
use tokenizer::tokenize;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error(transparent)]
    Parse(#[from] parser::Error),
}

/// Formats a valid VRL program into the canonical MVP style.
pub fn format(source: &str) -> Result<String, FormatError> {
    let _program = parser::parse(source)?;
    Ok(Formatter::new(tokenize(source)).format())
}

#[cfg(test)]
mod tests {
    use super::format;

    #[test]
    fn formats_indentation_operators_and_comments() {
        let source = "# keep\nif .ok{.value=1+2 # trailing\n}";
        let expected = "# keep\nif .ok {\n    .value = 1 + 2 # trailing\n}\n";

        assert_eq!(format(source).unwrap(), expected);
    }

    #[test]
    fn formatting_is_idempotent() {
        let source = "if .ok{.value=1+2}";
        let formatted = format(source).unwrap();

        assert_eq!(format(&formatted).unwrap(), formatted);
    }

    #[test]
    fn rejects_invalid_source() {
        assert!(format("a(r')").is_err());
    }

    #[test]
    fn keeps_braces_and_comment_markers_inside_literals() {
        let source = r#".message="{ # not a comment }" # a real comment"#;
        let expected = r#".message = "{ # not a comment }" # a real comment
"#;

        assert_eq!(format(source).unwrap(), expected);
    }

    #[test]
    fn preserves_empty_lines_and_inline_comments() {
        let source =
            ".first=1 # keep with first\n\n\n# separate section\n.second=2 # keep with second";
        let expected = ".first = 1 # keep with first\n\n\n# separate section\n.second = 2 # keep with second\n";

        assert_eq!(format(source).unwrap(), expected);
    }

    #[test]
    fn preserves_inline_comment_after_opening_brace() {
        let source = "if .ok { # explain condition\n.value=1\n}";
        let expected = "if .ok { # explain condition\n    .value = 1\n}\n";

        assert_eq!(format(source).unwrap(), expected);
    }
}
