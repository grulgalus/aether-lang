use crate::lexer::{Lexer, Token};
use crate::ast::{Program, Stmt, Expr};

pub struct Parser { lexer: Lexer, current_token: Token, peek_token: Token }
impl Parser {
    pub fn new(mut lexer: Lexer) -> Self { let current_token = lexer.next_token(); let peek_token = lexer.next_token(); Parser { lexer, current_token, peek_token } }
    pub fn next_token(&mut self) { self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token()); }
    pub fn parse_program(&mut self) -> Program { let mut program = Program { statements: Vec::new() }; while self.current_token != Token::EOF { if let Some(stmt) = self.parse_statement() { program.statements.push(stmt); } else { self.next_token(); } } program }
    fn parse_block(&mut self) -> Vec<Stmt> { let mut block = Vec::new(); self.next_token(); if self.current_token != Token::Symbol('{') { return block; } self.next_token(); while self.current_token != Token::Symbol('}') && self.current_token != Token::EOF { if let Some(stmt) = self.parse_statement() { block.push(stmt); } else { self.next_token(); } } self.next_token(); block }
    
    fn parse_statement(&mut self) -> Option<Stmt> { match &self.current_token { Token::Keyword(kw) if kw == "let" => self.parse_let(), Token::Keyword(kw) if kw == "return" => self.parse_return(), Token::Keyword(kw) if kw == "print" => self.parse_print(), Token::Keyword(kw) if kw == "fn" => self.parse_function(), Token::Keyword(kw) if kw == "actor" => self.parse_actor(), Token::Keyword(kw) if kw == "if" => self.parse_if(), Token::Keyword(kw) if kw == "while" => self.parse_while(), Token::Keyword(kw) if kw == "for" => self.parse_for(), Token::Keyword(kw) if kw == "import" => { self.next_token(); let file = match &self.current_token { Token::StringLiteral(s) => s.clone(), _ => return None }; self.next_token(); Some(Stmt::Import(file)) } Token::Identifier(name) => { if let Token::Operator(op) = &self.peek_token { if op == "=" { let n = name.clone(); self.next_token(); self.next_token(); let value = self.parse_expression()?; self.next_token(); return Some(Stmt::Assign { name: n, value }); } } let expr = self.parse_expression()?; self.next_token(); Some(Stmt::Expression(expr)) } _ => None, } }
    fn parse_if(&mut self) -> Option<Stmt> { self.next_token(); let condition = self.parse_expression()?; let consequence = self.parse_block(); let mut alternative = None; if let Token::Keyword(kw) = &self.current_token { if kw == "else" { alternative = Some(self.parse_block()); } } Some(Stmt::If { condition, consequence, alternative }) }
    fn parse_while(&mut self) -> Option<Stmt> { self.next_token(); let cond = self.parse_expression()?; let body = self.parse_block(); Some(Stmt::While { condition: cond, body }) }
    fn parse_for(&mut self) -> Option<Stmt> { self.next_token(); let variable = match &self.current_token { Token::Identifier(id) => id.clone(), _ => return None }; self.next_token(); self.next_token(); let iterable = self.parse_expression()?; self.next_token(); let body = self.parse_block(); Some(Stmt::For { variable, iterable, body }) }
    fn parse_let(&mut self) -> Option<Stmt> { self.next_token(); let name = match &self.current_token { Token::Identifier(id) => id.clone(), _ => return None }; self.next_token(); self.next_token(); let value = self.parse_expression()?; self.next_token(); Some(Stmt::Let { name, value }) }
    fn parse_actor(&mut self) -> Option<Stmt> { self.next_token(); let name = match &self.current_token { Token::Identifier(id) => id.clone(), _ => return None }; let methods = self.parse_block(); Some(Stmt::Actor { name, methods }) }
    fn parse_function(&mut self) -> Option<Stmt> { self.next_token(); let name = match &self.current_token { Token::Identifier(id) => id.clone(), _ => return None }; self.next_token(); self.next_token(); let body = self.parse_block(); Some(Stmt::Function { name, body }) }
    fn parse_print(&mut self) -> Option<Stmt> { self.next_token(); let has_parens = self.current_token == Token::Symbol('('); if has_parens { self.next_token(); } let value = self.parse_expression()?; self.next_token(); if has_parens && self.current_token == Token::Symbol(')') { self.next_token(); } Some(Stmt::Print { value }) }
    fn parse_return(&mut self) -> Option<Stmt> { self.next_token(); let value = self.parse_expression()?; self.next_token(); Some(Stmt::Return { value }) }
    
    // Zde voláme výpočet, začínáme na nejnižší prioritě 0
    fn parse_expression(&mut self) -> Option<Expr> { self.parse_expression_with_precedence(0) }
    
    // TOTO JE OPRAVA! CHYTRÁ MATEMATIKA (Prattův Parser)
    fn parse_expression_with_precedence(&mut self, precedence: u8) -> Option<Expr> {
        let mut left = match &self.current_token { 
            Token::Number(num) => Expr::Number(num.clone()), 
            Token::StringLiteral(s) => Expr::StringLit(s.clone()), 
            Token::Keyword(kw) if kw == "true" => Expr::Boolean(true), 
            Token::Keyword(kw) if kw == "false" => Expr::Boolean(false), 
            Token::Symbol('[') => { self.next_token(); let mut elements = Vec::new(); if self.current_token != Token::Symbol(']') { loop { if let Some(el) = self.parse_expression() { elements.push(el); } self.next_token(); if self.current_token == Token::Symbol(',') { self.next_token(); } else { break; } } } Expr::Array(elements) }, 
            Token::Symbol('{') => { self.next_token(); let mut pairs = Vec::new(); if self.current_token != Token::Symbol('}') { loop { let key = match &self.current_token { Token::Identifier(id) => id.clone(), Token::StringLiteral(s) => s.clone(), _ => break }; self.next_token(); if self.current_token == Token::Symbol(':') { self.next_token(); } if let Some(val) = self.parse_expression() { pairs.push((key, Box::new(val))); } self.next_token(); if self.current_token == Token::Symbol(',') { self.next_token(); } else { break; } } } Expr::Dict(pairs) }, 
            Token::Identifier(id) => { let name = id.clone(); if self.peek_token == Token::Symbol('(') { self.next_token(); self.next_token(); let mut args = Vec::new(); if self.current_token != Token::Symbol(')') { loop { if let Some(arg) = self.parse_expression() { args.push(arg); } self.next_token(); if self.current_token == Token::Symbol(',') { self.next_token(); } else { break; } } } Expr::Call { function: name, args } } else { Expr::Identifier(name) } }, 
            
            // OPRAVA ZÁVOREK (1 + 2) * 3
            Token::Symbol('(') => { 
                self.next_token(); 
                let expr = self.parse_expression()?; 
                if self.peek_token == Token::Symbol(')') { self.next_token(); } 
                expr 
            },
            _ => return None, 
        };
        
        loop {
            if self.peek_token == Token::Symbol('[') { self.next_token(); self.next_token(); let index = self.parse_expression()?; self.next_token(); left = Expr::Index { left: Box::new(left), index: Box::new(index) }; continue; }
            
            // KOUZLO: Určení priority! (Násobení má 6, Sčítání má 5)
            if let Token::Operator(op) = &self.peek_token { 
                let p = match op.as_str() { "||" => 1, "&&" => 2, "==" | "!=" => 3, "<" | ">" | "<=" | ">=" => 4, "+" | "-" => 5, "*" | "/" => 6, _ => 0 };
                
                // Pokud má operátor větší prioritu než to, co počítáme doteď, vypočítá se dřív!
                if p > precedence {
                    let operator = op.clone(); self.next_token(); self.next_token(); 
                    if let Some(right) = self.parse_expression_with_precedence(p) { 
                        left = Expr::BinaryOp { left: Box::new(left), operator, right: Box::new(right) }; 
                        continue; 
                    }
                }
            }
            break;
        }
        Some(left)
    }
}
