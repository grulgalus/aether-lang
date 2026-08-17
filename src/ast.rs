#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Identifier(String), Number(String), StringLit(String), Boolean(bool),
    Array(Vec<Expr>),
    Index { left: Box<Expr>, index: Box<Expr> },
    BinaryOp { left: Box<Expr>, operator: String, right: Box<Expr> },
    Call { function: String, args: Vec<Expr> },
}
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Let { name: String, value: Expr }, Assign { name: String, value: Expr },
    Expression(Expr), Return { value: Expr }, Print { value: Expr },
    If { condition: Expr, consequence: Vec<Stmt>, alternative: Option<Vec<Stmt>> },
    While { condition: Expr, body: Vec<Stmt> },
    Function { name: String, body: Vec<Stmt> }, Actor { name: String, methods: Vec<Stmt> },
}
#[derive(Debug, PartialEq)] pub struct Program { pub statements: Vec<Stmt> }
