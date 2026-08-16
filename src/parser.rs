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
        Parser {
            lexer,
            current_token,
            peek_token,
        }
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
                self.next_token(); // Přeskočí neznámé tokeny (např. volné znaky)
            }
        }
        
        program
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.current_token {
            Token::Keyword(ref kw) if kw == "let" => self.parse_let_statement(),
            Token::Keyword(ref kw) if kw == "return" => self.parse_return_statement(),
            Token::Keyword(ref kw) if kw == "fn" => self.parse_function_statement(),
            _ => None,
        }
    }

    fn parse_let_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // přeskočí 'let'
        
        let name = match &self.current_token {
            Token::Identifier(ident) => ident.clone(),
            _ => return None,
        };
        self.next_token(); // přeskočí jméno proměnné

        if self.current_token != Token::Operator("=".to_string()) {
            return None;
        }
        self.next_token(); // přeskočí '='

        let value = self.parse_expression()?;
        self.next_token(); // přeskočí hodnotu

        Some(Stmt::Let { name, value })
    }

    fn parse_return_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // přeskočí 'return'
        
        let value = self.parse_expression()?;
        self.next_token(); // přeskočí hodnotu
        
        Some(Stmt::Return { value })
    }

    fn parse_function_statement(&mut self) -> Option<Stmt> {
        self.next_token(); // přeskočí 'fn'
        
        let name = match &self.current_token {
            Token::Identifier(ident) => ident.clone(),
            _ => return None,
        };
        self.next_token(); // přeskočí jméno funkce
        
        // Zpracování parametrů funkce: ()
        if self.current_token != Token::Symbol('(') { return None; }
        self.next_token();
        if self.current_token != Token::Symbol(')') { return None; }
        self.next_token();
        
        // Zpracování těla funkce: { ... }
        if self.current_token != Token::Symbol('{') { return None; }
        self.next_token();
        
        let mut body = Vec::new();
        while self.current_token != Token::Symbol('}') && self.current_token != Token::EOF {
            if let Some(stmt) = self.parse_statement() {
                body.push(stmt);
            } else {
                self.next_token();
            }
        }
        
        self.next_token(); // přeskočí '}'
        
        Some(Stmt::Function { name, body })
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        match &self.current_token {
            Token::Number(num) => Some(Expr::Number(num.clone())),
            Token::StringLiteral(s) => Some(Expr::StringLit(s.clone())),
            Token::Identifier(id) => Some(Expr::Identifier(id.clone())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_function_parsing() {
        let input = "fn setup() { let x = 42 return x }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 1);
        
        if let Stmt::Function { name, body } = &program.statements[0] {
            assert_eq!(name, "setup");
            assert_eq!(body.len(), 2);
            assert_eq!(body[0], Stmt::Let { name: "x".to_string(), value: Expr::Number("42".to_string()) });
            assert_eq!(body[1], Stmt::Return { value: Expr::Identifier("x".to_string()) });
        } else {
            panic!("Očekávala se funkce!");
        }
    }
}
