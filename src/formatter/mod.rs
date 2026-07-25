//! Canonical source formatting for VRL.
//!
//! Formatting validates input through the parser before applying source-aware
//! whitespace rules. The scanner keeps comments and literals intact so the
//! formatter never has to reconstruct them from compiler state.

use crate::parser;

const INDENT: &str = "    ";
const LINE_WIDTH: usize = 100;

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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Word(String),
    Literal(String),
    Comment(String),
    Newline,
    Open(char),
    Close(char),
    Comma,
    Colon,
    Semicolon,
    Operator(String),
    Dot,
    Bang,
    Ampersand,
    Percent,
    Arrow,
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((start, ch)) = chars.next() {
        match ch {
            '#' => {
                let end = consume_until_newline(source, &mut chars);
                tokens.push(Token::Comment(source[start..end].to_owned()));
            }
            '\n' => tokens.push(Token::Newline),
            ch if ch.is_whitespace() => {}
            '"' => {
                let end = consume_quoted(source, &mut chars, '"');
                tokens.push(Token::Literal(source[start..end].to_owned()));
            }
            'r' | 's' | 't' if chars.peek().is_some_and(|(_, next)| *next == '\'') => {
                let _quote = chars.next();
                let end = consume_quoted(source, &mut chars, '\'');
                tokens.push(Token::Literal(source[start..end].to_owned()));
            }
            '(' | '[' | '{' => tokens.push(Token::Open(ch)),
            ')' | ']' | '}' => tokens.push(Token::Close(ch)),
            ',' => tokens.push(Token::Comma),
            ':' => tokens.push(Token::Colon),
            ';' => tokens.push(Token::Semicolon),
            '.' => tokens.push(Token::Dot),
            '%' => tokens.push(Token::Percent),
            '&' if chars.peek().is_some_and(|(_, next)| *next == '&') => {
                let _next = chars.next();
                tokens.push(Token::Operator("&&".to_owned()));
            }
            '&' => tokens.push(Token::Ampersand),
            '-' if chars.peek().is_some_and(|(_, next)| *next == '>') => {
                let _next = chars.next();
                tokens.push(Token::Arrow);
            }
            '!' if chars.peek().is_some_and(|(_, next)| *next == '=') => {
                let _next = chars.next();
                tokens.push(Token::Operator("!=".to_owned()));
            }
            '!' => tokens.push(Token::Bang),
            '=' | '|' | '+' | '-' | '*' | '/' | '<' | '>' | '?' => {
                let end = consume_operator(start, ch, &mut chars);
                tokens.push(Token::Operator(source[start..end].to_owned()));
            }
            _ => {
                let end = consume_word(source, &mut chars);
                tokens.push(Token::Word(source[start..end].to_owned()));
            }
        }
    }

    tokens
}

fn consume_until_newline(
    source: &str,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> usize {
    let mut end = source.len();
    while let Some((index, ch)) = chars.peek().copied() {
        if ch == '\n' {
            end = index;
            break;
        }
        let _next = chars.next();
    }
    end
}

fn consume_quoted(
    source: &str,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    quote: char,
) -> usize {
    let mut escaped = false;
    let mut end = source.len();
    for (index, ch) in chars.by_ref() {
        end = index + ch.len_utf8();
        if ch == quote && !escaped {
            break;
        }
        escaped = ch == '\\' && !escaped;
        if ch != '\\' {
            escaped = false;
        }
    }
    end
}

fn consume_operator(
    start: usize,
    first: char,
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> usize {
    let Some((index, next)) = chars.peek().copied() else {
        return start + first.len_utf8();
    };
    let pair = matches!(
        (first, next),
        ('=', '=') | ('|', '=') | ('|', '|') | ('<', '=') | ('>', '=') | ('?', '?')
    );
    if pair {
        let _next = chars.next();
        index + next.len_utf8()
    } else {
        start + first.len_utf8()
    }
}

fn consume_word(source: &str, chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> usize {
    let mut end = source.len();
    while let Some((index, ch)) = chars.peek().copied() {
        if ch.is_whitespace() || "#()[]{} ,:;.%!&=|+-*/<>?\"".contains(ch) {
            end = index;
            break;
        }
        let _next = chars.next();
    }
    end
}

struct Formatter {
    tokens: Vec<Token>,
    output: String,
    indent: usize,
    line_start: bool,
    closure_pipes: usize,
}

impl Formatter {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            output: String::new(),
            indent: 0,
            line_start: true,
            closure_pipes: 0,
        }
    }

    fn format(mut self) -> String {
        for index in 0..self.tokens.len() {
            let previous = index
                .checked_sub(1)
                .and_then(|index| self.tokens.get(index))
                .cloned();
            let next = self.tokens.get(index + 1).cloned();
            match self.tokens[index].clone() {
                Token::Word(word) | Token::Literal(word) => self.word(&word, previous.as_ref()),
                Token::Comment(comment) => self.comment(&comment),
                Token::Newline => self.newline(),
                Token::Open(delimiter) => self.open(delimiter, next.as_ref()),
                Token::Close(delimiter) => self.close(delimiter),
                Token::Comma => {
                    self.trim_space();
                    self.output.push(',');
                    self.space();
                }
                Token::Colon => {
                    self.trim_space();
                    self.output.push(':');
                    self.space();
                }
                Token::Semicolon => {
                    self.trim_space();
                    self.output.push(';');
                    self.newline();
                }
                Token::Operator(operator) => self.operator(&operator),
                Token::Dot => self.dot(previous.as_ref()),
                Token::Bang => {
                    self.trim_space();
                    self.write("!");
                }
                Token::Ampersand => {
                    self.trim_space();
                    self.write("&");
                }
                Token::Percent => {
                    self.trim_space();
                    self.write("%");
                }
                Token::Arrow => {
                    self.space();
                    self.write("->");
                    self.space();
                    self.closure_pipes = 2;
                }
            }
        }

        self.trim_space();
        if !self.output.is_empty() {
            self.newline();
        }
        self.output
    }

    fn word(&mut self, word: &str, previous: Option<&Token>) {
        let joins_previous = matches!(
            previous,
            Some(Token::Dot | Token::Bang | Token::Ampersand | Token::Percent)
        ) || (self.closure_pipes == 1
            && matches!(previous, Some(Token::Operator(pipe)) if pipe == "|"));
        if !self.line_start
            && !joins_previous
            && !self.output.ends_with(' ')
            && !self.output.ends_with('\n')
            && !self.output.ends_with('(')
            && !self.output.ends_with('[')
        {
            self.space();
        }
        self.write(word);
    }

    fn comment(&mut self, comment: &str) {
        if !self.line_start {
            self.space();
        }
        self.write(comment);
        self.newline();
    }

    fn open(&mut self, delimiter: char, next: Option<&Token>) {
        self.trim_space();
        if delimiter == '{' && !self.line_start {
            self.space();
        }
        self.write(&delimiter.to_string());
        if delimiter == '{' && !matches!(next, Some(Token::Close('}'))) {
            self.indent += 1;
            self.newline();
        }
    }

    fn close(&mut self, delimiter: char) {
        if delimiter == '}' && self.indent > 0 {
            self.indent -= 1;
            self.newline();
        }
        self.trim_space();
        self.write(&delimiter.to_string());
    }

    fn operator(&mut self, operator: &str) {
        if operator == "|" && self.closure_pipes > 0 {
            if self.closure_pipes == 2 {
                self.space();
            } else {
                self.trim_space();
            }
            self.write("|");
            self.closure_pipes -= 1;
        } else {
            self.space();
            self.write(operator);
            self.space();
        }
    }

    fn dot(&mut self, previous: Option<&Token>) {
        if matches!(previous, Some(Token::Word(word)) if word == "if") {
            self.space();
        } else {
            self.trim_space();
        }
        self.write(".");
    }

    fn write(&mut self, value: &str) {
        if self.line_start {
            for _ in 0..self.indent {
                self.output.push_str(INDENT);
            }
            self.line_start = false;
        }
        self.output.push_str(value);
    }

    fn space(&mut self) {
        if !self.line_start && !self.output.ends_with(' ') && !self.output.ends_with('\n') {
            if self.column() >= LINE_WIDTH {
                self.newline();
            } else {
                self.output.push(' ');
            }
        }
    }

    fn trim_space(&mut self) {
        while self.output.ends_with(' ') {
            let _removed = self.output.pop();
        }
    }

    fn newline(&mut self) {
        self.trim_space();
        if !self.line_start {
            self.output.push('\n');
            self.line_start = true;
        }
    }

    fn column(&self) -> usize {
        self.output
            .rsplit_once('\n')
            .map_or(self.output.len(), |(_, line)| line.len())
    }
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
}
