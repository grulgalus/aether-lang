#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier(String),
    Number(String),
    StringLit(String),
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Let { name: String, value: Expr },
    Return { value: Expr },
    Function { name: String, body: Vec<Stmt> },
    Actor { name: String, methods: Vec<Stmt> },
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
