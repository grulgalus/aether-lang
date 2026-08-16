#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier(String),
    Number(String),
    StringLit(String),
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Let { name: String, value: Expr },
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
