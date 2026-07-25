use super::tokenizer::Token;

const INDENT: &str = "    ";
const LINE_WIDTH: usize = 100;

pub(super) struct Formatter {
    tokens: Vec<Token>,
    output: String,
    indent: usize,
    line_start: bool,
    closure_pipes: usize,
}

impl Formatter {
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            output: String::new(),
            indent: 0,
            line_start: true,
            closure_pipes: 0,
        }
    }

    pub(super) fn format(mut self) -> String {
        for index in 0..self.tokens.len() {
            let previous = index
                .checked_sub(1)
                .and_then(|index| self.tokens.get(index))
                .cloned();
            let next = self.tokens.get(index + 1).cloned();
            match self.tokens[index].clone() {
                Token::Word(word) | Token::Literal(word) => self.word(&word, previous.as_ref()),
                Token::Comment(comment) => self.comment(&comment, next.as_ref()),
                Token::Newline => self
                    .source_newline(previous.is_none() || matches!(previous, Some(Token::Newline))),
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

    fn comment(&mut self, comment: &str, next: Option<&Token>) {
        if !self.line_start {
            self.space();
        }
        self.write(comment);
        if !matches!(next, Some(Token::Newline)) {
            self.newline();
        }
    }

    fn open(&mut self, delimiter: char, next: Option<&Token>) {
        self.trim_space();
        if delimiter == '{' && !self.line_start {
            self.space();
        }
        self.write(&delimiter.to_string());
        if delimiter == '{' && !matches!(next, Some(Token::Close('}')) | Some(Token::Comment(_))) {
            self.indent += 1;
            self.newline();
        } else if delimiter == '{' && matches!(next, Some(Token::Comment(_))) {
            self.indent += 1;
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

    fn source_newline(&mut self, preserves_empty_line: bool) {
        self.trim_space();
        if self.line_start && preserves_empty_line {
            self.output.push('\n');
        } else if !self.line_start {
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
