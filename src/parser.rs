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
                // Pokud narazíme na něco, co ještě neumíme, zatím to přeskočíme
                self.next_token();
            }
        }
        
        program
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.current_token {
            Token::Keyword(ref kw) if kw == "let" => self.parse_let_statement(),
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

        let value = match &self.current_token {
            Token::Number(num) => Expr::Number(num.clone()),
            Token::StringLiteral(s) => Expr::StringLit(s.clone()),
            Token::Identifier(id) => Expr::Identifier(id.clone()),
            _ => return None,
        };
        self.next_token(); // přeskočí hodnotu

        Some(Stmt::Let { name, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_let_statements() {
        let input = "let x = 42 let name = \"Aether\" let y = x";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 3);
        
        assert_eq!(
            program.statements[0],
            Stmt::Let { name: "x".to_string(), value: Expr::Number("42".to_string()) }
        );
        assert_eq!(
            program.statements[1],
            Stmt::Let { name: "name".to_string(), value: Expr::StringLit("Aether".to_string()) }
        );
        assert_eq!(
            program.statements[2],
            Stmt::Let { name: "y".to_string(), value: Expr::Identifier("x".to_string()) }
        );
    }
}
