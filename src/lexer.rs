#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    StringLiteral(String),
    Number(String),
    Operator(String),
    Symbol(char),
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.position >= self.input.len() { return Token::EOF; }

        let ch = self.input[self.position];

        if ch.is_alphabetic() || ch == '_' { return self.read_identifier_or_keyword(); }
        if ch.is_ascii_digit() { return self.read_number(); }
        if ch == '"' { return self.read_string_literal(); }

        let next_ch = if self.position + 1 < self.input.len() { self.input[self.position + 1] } else { '\0' };

        match (ch, next_ch) {
            ('-', '>') | ('=', '=') | ('!', '=') | ('<', '=') | ('>', '=') => {
                self.position += 2;
                return Token::Operator(format!("{}{}", ch, next_ch));
            }
            _ => {}
        }

        self.position += 1;
        if "+-*/=!<>".contains(ch) { return Token::Operator(ch.to_string()); }
        Token::Symbol(ch)
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() { self.position += 1; }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_') {
            self.position += 1;
        }
        let text: String = self.input[start..self.position].iter().collect();
        match text.as_str() {
            "actor" | "fn" | "let" | "return" | "match" | "Result" | "Ok" | "Err" | "print" => Token::Keyword(text),
            _ => Token::Identifier(text),
        }
    }

    fn read_string_literal(&mut self) -> Token {
        self.position += 1;
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position] != '"' { self.position += 1; }
        let text: String = self.input[start..self.position].iter().collect();
        self.position += 1;
        Token::StringLiteral(text)
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_ascii_digit() || self.input[self.position] == '.') {
            self.position += 1;
        }
        let text: String = self.input[start..self.position].iter().collect();
        Token::Number(text)
    }
}
