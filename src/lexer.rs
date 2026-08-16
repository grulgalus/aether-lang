#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    StringLiteral(String),
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

        if self.position >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.input[self.position];

        if ch.is_alphabetic() {
            return self.read_identifier_or_keyword();
        }

        if ch == '"' {
            return self.read_string_literal();
        }

        // Basic symbol handling
        self.position += 1;
        Token::Symbol(ch)
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_') {
            self.position += 1;
        }
        
        let text: String = self.input[start..self.position].iter().collect();
        
        // Basic keyword matching
        match text.as_str() {
            "actor" | "fn" | "let" | "return" | "match" => Token::Keyword(text),
            _ => Token::Identifier(text),
        }
    }

    fn read_string_literal(&mut self) -> Token {
        self.position += 1; // skip opening quote
        let start = self.position;
        
        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }
        
        let text: String = self.input[start..self.position].iter().collect();
        self.position += 1; // skip closing quote
        
        Token::StringLiteral(text)
    }
}
