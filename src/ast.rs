#[derive(Debug, PartialEq)]
pub enum Expr {
    Identifier(String),
    Number(String),
    StringLit(String),
    BinaryOp { left: Box<Expr>, operator: String, right: Box<Expr> },
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Let { name: String, value: Expr },
    Return { value: Expr },
    Print { value: Expr },
    Function { name: String, body: Vec<Stmt> },
    Actor { name: String, methods: Vec<Stmt> },
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
