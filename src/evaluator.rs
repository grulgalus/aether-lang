use std::collections::HashMap;
use crate::ast::{Program, Stmt, Expr};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Object { Number(f64), StringObj(String), Boolean(bool), Array(Vec<Object>), Dict(HashMap<String, Object>), Null }

pub struct Environment { pub store: HashMap<String, Object>, pub verbose: bool, pub output: Vec<String> }
impl Environment {
    pub fn new(verbose: bool) -> Self { Environment { store: HashMap::new(), verbose, output: Vec::new() } }
    pub fn set(&mut self, name: String, val: Object) { self.store.insert(name, val); }
    pub fn get(&self, name: &str) -> Option<Object> { self.store.get(name).cloned() }
}

pub fn eval_program(program: &Program, env: &mut Environment) -> Object { eval_block(&program.statements, env) }

fn eval_block(statements: &[Stmt], env: &mut Environment) -> Object {
    let mut result = Object::Null;
    for stmt in statements { result = eval_statement(stmt, env); if let Stmt::Return { .. } = stmt { break; } }
    result
}

fn eval_statement(stmt: &Stmt, env: &mut Environment) -> Object {
    match stmt {
        Stmt::Import(path) => { if env.verbose { println!("📦 Nacitam modul: {}", path); } if let Ok(content) = std::fs::read_to_string(path) { let lexer = Lexer::new(&content); let mut parser = Parser::new(lexer); let program = parser.parse_program(); eval_block(&program.statements, env); } Object::Null }
        Stmt::Let { name, value } | Stmt::Assign { name, value } => { let val = eval_expression(value, env); env.set(name.clone(), val); Object::Null }
        Stmt::Expression(expr) => { eval_expression(expr, env); Object::Null }
        Stmt::Return { value } => eval_expression(value, env),
        Stmt::Print { value } => {
            fn format_obj(obj: &Object) -> String { match obj { Object::Number(n) => n.to_string(), Object::StringObj(s) => s.clone(), Object::Boolean(b) => b.to_string(), Object::Array(arr) => { let strings: Vec<String> = arr.iter().map(|x| match x { Object::StringObj(s) => format!("\"{}\"", s), _ => format_obj(x) }).collect(); format!("[{}]", strings.join(", ")) } Object::Dict(d) => { let pairs: Vec<String> = d.iter().map(|(k, v)| format!("{}: {}", k, format_obj(v))).collect(); format!("{{{}}}", pairs.join(", ")) } Object::Null => "null".to_string(), } }
            let text = format_obj(&eval_expression(value, env)); println!("{}", text); env.output.push(text); Object::Null
        }
        Stmt::Actor { methods, .. } => eval_block(methods, env),
        Stmt::Function { body, .. } => eval_block(body, env),
        Stmt::If { condition, consequence, alternative } => { let cond = eval_expression(condition, env); let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true }; if is_truthy { eval_block(consequence, env) } else if let Some(alt) = alternative { eval_block(alt, env) } else { Object::Null } }
        Stmt::While { condition, body } => { let mut res = Object::Null; loop { let cond = eval_expression(condition, env); let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true }; if !is_truthy { break; } res = eval_block(body, env); } res }
        Stmt::For { variable, iterable, body } => { let iter_val = eval_expression(iterable, env); let mut res = Object::Null; if let Object::Array(arr) = iter_val { for item in arr { env.set(variable.clone(), item); res = eval_block(body, env); } } res }
    }
}

fn eval_expression(expr: &Expr, env: &Environment) -> Object {
    match expr {
        Expr::Number(s) => Object::Number(s.parse().unwrap_or(0.0)), Expr::StringLit(s) => Object::StringObj(s.clone()), Expr::Boolean(b) => Object::Boolean(*b),
        Expr::Array(elements) => { let mut evaluated = Vec::new(); for el in elements { evaluated.push(eval_expression(el, env)); } Object::Array(evaluated) }
        Expr::Dict(pairs) => { let mut map = HashMap::new(); for (k, v) in pairs { map.insert(k.clone(), eval_expression(v, env)); } Object::Dict(map) }
        Expr::Index { left, index } => { let left_val = eval_expression(left, env); let index_val = eval_expression(index, env); if let (Object::Array(arr), Object::Number(i)) = (&left_val, &index_val) { let idx = *i as usize; if idx < arr.len() { return arr[idx].clone(); } } if let (Object::Dict(map), Object::StringObj(k)) = (&left_val, &index_val) { if let Some(val) = map.get(k) { return val.clone(); } } Object::Null }
        Expr::Identifier(s) => { if s == "input" { let mut line = String::new(); std::io::stdin().read_line(&mut line).unwrap_or(0); return Object::StringObj(line.trim().to_string()); } env.get(s).unwrap_or(Object::Null) }
        Expr::Call { function, args } => {
            let mut eval_args = Vec::new(); for arg in args { eval_args.push(eval_expression(arg, env)); }
            match function.as_str() {
                "type" => { if let Some(arg) = eval_args.get(0) { return Object::StringObj(match arg { Object::Number(_) => "number", Object::StringObj(_) => "string", Object::Boolean(_) => "boolean", Object::Array(_) => "array", Object::Dict(_) => "dict", Object::Null => "null" }.to_string()); } Object::Null },
                "time" => { let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(); return Object::Number(t as f64); },
                "keys" => { if let Some(Object::Dict(map)) = eval_args.get(0) { return Object::Array(map.keys().map(|k| Object::StringObj(k.clone())).collect()); } Object::Null },
                "len" => { if let Some(Object::StringObj(s)) = eval_args.get(0) { return Object::Number(s.len() as f64); } if let Some(Object::Array(arr)) = eval_args.get(0) { return Object::Number(arr.len() as f64); } Object::Null },
                "push" => { if let (Some(Object::Array(mut arr)), Some(val)) = (eval_args.get(0).cloned(), eval_args.get(1).cloned()) { arr.push(val); return Object::Array(arr); } Object::Null },
                "rand" => { let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(); return Object::Number((t % 100) as f64); }
                _ => Object::Null
            }
        }
        Expr::BinaryOp { left, operator, right } => { let left_val = eval_expression(left, env); let right_val = eval_expression(right, env); if let (Object::Number(l), Object::Number(r)) = (&left_val, &right_val) { match operator.as_str() { "+" => Object::Number(l + r), "-" => Object::Number(l - r), "*" => Object::Number(l * r), "/" => if *r != 0.0 { Object::Number(l / r) } else { Object::Null }, "<" => Object::Boolean(l < r), ">" => Object::Boolean(l > r), "==" => Object::Boolean(l == r), "!=" => Object::Boolean(l != r), "<=" => Object::Boolean(l <= r), ">=" => Object::Boolean(l >= r), _ => Object::Null, } } else if let (Object::StringObj(l), Object::StringObj(r)) = (&left_val, &right_val) { if operator == "+" { Object::StringObj(format!("{}{}", l, r)) } else { Object::Null } } else { Object::Null } }
    }
}
