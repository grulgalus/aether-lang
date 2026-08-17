use std::collections::HashMap;
use crate::ast::{Program, Stmt, Expr};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Object { Number(f64), StringObj(String), Boolean(bool), Array(Vec<Object>), Dict(HashMap<String, Object>), Null, Error(String) }

impl Object {
    pub fn to_string_val(&self) -> String {
        match self {
            Object::Number(n) => n.to_string(), Object::StringObj(s) => s.clone(), Object::Boolean(b) => b.to_string(),
            Object::Array(arr) => { let strings: Vec<String> = arr.iter().map(|x| x.to_string_val()).collect(); format!("[{}]", strings.join(", ")) },
            Object::Dict(d) => { let pairs: Vec<String> = d.iter().map(|(k, v)| format!("{}: {}", k, v.to_string_val())).collect(); format!("{{{}}}", pairs.join(", ")) },
            Object::Null => "null".to_string(),
            Object::Error(e) => format!("🛑 {}", e),
        }
    }
}

pub struct Environment { pub store: HashMap<String, Object>, pub output: Vec<String> }
impl Environment {
    pub fn new() -> Self { Environment { store: HashMap::new(), output: Vec::new() } }
    pub fn set(&mut self, name: String, val: Object) { self.store.insert(name, val); }
    pub fn get(&self, name: &str) -> Option<Object> { self.store.get(name).cloned() }
}

// 1. FÁZE: STATICKÁ KONTROLA (CHECK)
fn check_program(program: &Program) -> Result<(), String> {
    // Tady můžeme v budoucnu přidat kontrolu syntaxe před spuštěním
    Ok(())
}

// 2. FÁZE: OPTIMALIZACE (CONSTANT FOLDING)
fn optimize_expr(expr: Expr) -> Expr {
    match expr {
        Expr::BinaryOp { left, operator, right } => {
            let l = optimize_expr(*left);
            let r = optimize_expr(*right);
            if let (Expr::Number(n1), Expr::Number(n2)) = (&l, &r) {
                let v1 = n1.parse::<f64>().unwrap_or(0.0);
                let v2 = n2.parse::<f64>().unwrap_or(0.0);
                return match operator.as_str() {
                    "+" => Expr::Number((v1 + v2).to_string()),
                    "*" => Expr::Number((v1 * v2).to_string()),
                    _ => Expr::BinaryOp { left: Box::new(l), operator, right: Box::new(r) }
                };
            }
            Expr::BinaryOp { left: Box::new(l), operator, right: Box::new(r) }
        },
        _ => expr
    }
}

// 3. FÁZE: VIRTUÁLNÍ BĚH V PAMĚTI (RUN)
pub fn eval_program(program: &Program, env: &mut Environment) -> Object {
    if let Err(e) = check_program(program) {
        let err = Object::Error(format!("Chyba analýzy: {}", e));
        env.output.push(err.to_string_val());
        return err;
    }
    
    let mut result = Object::Null;
    for stmt in &program.statements {
        result = eval_statement(stmt, env);
        if let Object::Error(e) = result {
            let msg = format!("🛑 KRITICKÁ CHYBA: {}", e);
            env.output.push(msg);
            return Object::Error(e);
        }
    }
    result
}

fn eval_statement(stmt: &Stmt, env: &mut Environment) -> Object {
    match stmt {
        Stmt::Let { name, value } | Stmt::Assign { name, value } => {
            let val = eval_expression(value, env);
            if let Object::Error(_) = val { return val; }
            env.set(name.clone(), val);
            Object::Null
        }
        Stmt::Print { value } => {
            let val = eval_expression(value, env);
            if let Object::Error(_) = val { return val; }
            let text = val.to_string_val();
            env.output.push(text);
            Object::Null
        }
        Stmt::For { variable, iterable, body } => {
            let iter_val = eval_expression(iterable, env);
            if let Object::Array(arr) = iter_val {
                for item in arr {
                    env.set(variable.clone(), item);
                    let res = eval_block(body, env);
                    if let Object::Error(_) = res { return res; }
                }
                Object::Null
            } else { Object::Error("For vyžaduje pole".into()) }
        }
        Stmt::Expression(expr) => eval_expression(expr, env),
        _ => Object::Null
    }
}

fn eval_block(stmts: &[Stmt], env: &mut Environment) -> Object {
    let mut res = Object::Null;
    for s in stmts {
        res = eval_statement(s, env);
        if let Object::Error(_) = res { return res; }
    }
    res
}

fn eval_expression(expr: &Expr, env: &Environment) -> Object {
    match expr {
        Expr::Number(n) => Object::Number(n.parse().unwrap_or(0.0)),
        Expr::StringLit(s) => Object::StringObj(s.clone()),
        Expr::Identifier(s) => env.get(s).unwrap_or(Object::Error(format!("Neznámá proměnná '{}'", s))),
        Expr::Array(els) => {
            let mut arr = Vec::new();
            for el in els { arr.push(eval_expression(el, env)); }
            Object::Array(arr)
        }
        Expr::BinaryOp { left, operator, right } => {
            let l = eval_expression(left, env);
            let r = eval_expression(right, env);
            if let (Object::Number(v1), Object::Number(v2)) = (l, r) {
                match operator.as_str() {
                    "+" => Object::Number(v1 + v2),
                    "-" => Object::Number(v1 - v2),
                    "*" => Object::Number(v1 * v2),
                    "/" => if v2 != 0.0 { Object::Number(v1 / v2) } else { Object::Error("Dělení nulou".into()) },
                    _ => Object::Error("Neznámý operátor".into())
                }
            } else { Object::Error("Matematika vyžaduje čísla".into()) }
        }
        _ => Object::Null
    }
}
