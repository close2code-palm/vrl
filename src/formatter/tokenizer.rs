#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Token {
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

pub(super) fn tokenize(source: &str) -> Vec<Token> {
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
