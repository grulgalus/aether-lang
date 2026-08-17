use crate::lexer::{Lexer, Token};
use crate::ast::{Program, Stmt, Expr};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser { lexer, current_token, peek_token }
    }

    pub fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program { statements: Vec::new() };
        while self.current_token != Token::EOF {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            } else {
                self.next_token();
            }
        }
        program
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.current_token {
            Token::Keyword(ref kw) if kw == "let" => self.parse_let_statement(),
            Token::Keyword(ref kw) if kw == "return" => self.parse_return_statement(),
            Token::Keyword(ref kw) if kw == "print" => self.parse_print_statement(),
            Token::Keyword(ref kw) if kw == "fn" => self.parse_function_statement(),
            Token::Keyword(ref kw) if kw == "actor" => self.parse_actor_statement(),
            _ => None,
        }
    }

    fn parse_print_statement(&mut self) -> Option<Stmt> {
        self.next_token();
        let has_parens = self.current_token == Token::Symbol('(');
        if has_parens { self.next_token(); }

        let value = self.parse_expression()?;
        self.next_token();

        if has_parens && self.current_token == Token::Symbol(')') {
            self.next_token();
        }

        Some(Stmt::Print { value })
    }

    fn parse_actor_statement(&mut self) -> Option<Stmt> {
        self.next_token();
        let name = match &self.current_token { Token::Identifier(ident) => ident.clone(), _ => return None, };
        self.next_token();
        if self.current_token != Token::Symbol('{') { return None; }
        self.next_token();
        let mut methods = Vec::new();
        while self.current_token != Token::Symbol('}') && self.current_token != Token::EOF {
            if let Some(stmt) = self.parse_statement() { methods.push(stmt); } else { self.next_token(); }
        }
        self.next_token();
        Some(Stmt::Actor { name, methods })
    }

    fn parse_let_statement(&mut self) -> Option<Stmt> {
        self.next_token();
        let name = match &self.current_token { Token::Identifier(ident) => ident.clone(), _ => return None, };
        self.next_token();
        if self.current_token != Token::Operator("=".to_string()) { return None; }
        self.next_token();
        let value = self.parse_expression()?;
        self.next_token();
        Some(Stmt::Let { name, value })
    }

    fn parse_return_statement(&mut self) -> Option<Stmt> {
        self.next_token();
        let value = self.parse_expression()?;
        self.next_token();
        Some(Stmt::Return { value })
    }

    fn parse_function_statement(&mut self) -> Option<Stmt> {
        self.next_token();
        let name = match &self.current_token { Token::Identifier(ident) => ident.clone(), _ => return None, };
        self.next_token();
        if self.current_token != Token::Symbol('(') { return None; }
        self.next_token();
        if self.current_token != Token::Symbol(')') { return None; }
        self.next_token();
        if self.current_token != Token::Symbol('{') { return None; }
        self.next_token();
        let mut body = Vec::new();
        while self.current_token != Token::Symbol('}') && self.current_token != Token::EOF {
            if let Some(stmt) = self.parse_statement() { body.push(stmt); } else { self.next_token(); }
        }
        self.next_token();
        Some(Stmt::Function { name, body })
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        // Nejprve získáme levou stranu (číslo, proměnnou, text)
        let left = match &self.current_token {
            Token::Number(num) => Expr::Number(num.clone()),
            Token::StringLiteral(s) => Expr::StringLit(s.clone()),
            Token::Identifier(id) => Expr::Identifier(id.clone()),
            _ => return None,
        };

        // Podíváme se, jestli za ním nenásleduje matematický operátor
        if let Token::Operator(op) = &self.peek_token {
            if "+-*/".contains(op) {
                let operator = op.clone();
                self.next_token(); // přesuneme se z levé strany na operátor
                self.next_token(); // přesuneme se z operátoru na pravou stranu
                
                let right = match &self.current_token {
                    Token::Number(num) => Expr::Number(num.clone()),
                    Token::StringLiteral(s) => Expr::StringLit(s.clone()),
                    Token::Identifier(id) => Expr::Identifier(id.clone()),
                    _ => return None,
                };
                
                return Some(Expr::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                });
            }
        }

        Some(left)
    }
}
