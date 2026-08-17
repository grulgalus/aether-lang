use std::collections::HashMap;
use crate::ast::{Program, Stmt, Expr};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[derive(Debug, Clone, PartialEq)]
pub enum Object { Number(f64), StringObj(String), Boolean(bool), Array(Vec<Object>), Null }

// Přidali jsme 'verbose: bool', aby mozek věděl, jestli smí mluvit!
pub struct Environment { pub store: HashMap<String, Object>, pub verbose: bool }
impl Environment {
    pub fn new(verbose: bool) -> Self { Environment { store: HashMap::new(), verbose } }
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
        Stmt::Import(path) => {
            // TADY JE TA MAGIE: Kompilátor vypíše info o importu JEN KDYŽ NEDRŽÍ PUSU!
            if env.verbose {
                println!("📦 Nacitam importovany modul: {}", path);
            }
            if let Ok(content) = std::fs::read_to_string(path) {
                let lexer = Lexer::new(&content);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program();
                eval_block(&program.statements, env);
            } else {
                if env.verbose { println!("[CHYBA] Nelze najit a nacist modul: {}", path); }
            }
            Object::Null
        }
        Stmt::Let { name, value } | Stmt::Assign { name, value } => { let val = eval_expression(value, env); env.set(name.clone(), val); Object::Null }
        Stmt::Expression(expr) => { eval_expression(expr, env); Object::Null }
        Stmt::Return { value } => eval_expression(value, env),
        Stmt::Print { value } => {
            fn format_obj(obj: &Object) -> String { match obj { Object::Number(n) => n.to_string(), Object::StringObj(s) => s.clone(), Object::Boolean(b) => b.to_string(), Object::Array(arr) => { let strings: Vec<String> = arr.iter().map(|x| match x { Object::StringObj(s) => format!("\"{}\"", s), _ => format_obj(x) }).collect(); format!("[{}]", strings.join(", ")) } Object::Null => "null".to_string(), } }
            println!("{}", format_obj(&eval_expression(value, env))); Object::Null
        }
        Stmt::Actor { methods, .. } => eval_block(methods, env),
        Stmt::Function { body, .. } => eval_block(body, env),
        Stmt::If { condition, consequence, alternative } => { let cond = eval_expression(condition, env); let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true }; if is_truthy { eval_block(consequence, env) } else if let Some(alt) = alternative { eval_block(alt, env) } else { Object::Null } }
        Stmt::While { condition, body } => { let mut res = Object::Null; loop { let cond = eval_expression(condition, env); let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true }; if !is_truthy { break; } res = eval_block(body, env); } res }
    }
}

fn eval_expression(expr: &Expr, env: &Environment) -> Object {
    match expr {
        Expr::Number(s) => Object::Number(s.parse().unwrap_or(0.0)), Expr::StringLit(s) => Object::StringObj(s.clone()), Expr::Boolean(b) => Object::Boolean(*b),
        Expr::Array(elements) => { let mut evaluated = Vec::new(); for el in elements { evaluated.push(eval_expression(el, env)); } Object::Array(evaluated) }
        Expr::Index { left, index } => { let left_val = eval_expression(left, env); let index_val = eval_expression(index, env); if let (Object::Array(arr), Object::Number(i)) = (left_val, index_val) { let idx = i as usize; if idx < arr.len() { return arr[idx].clone(); } } Object::Null }
        Expr::Identifier(s) => { if s == "input" { let mut line = String::new(); std::io::stdin().read_line(&mut line).unwrap_or(0); return Object::StringObj(line.trim().to_string()); } env.get(s).unwrap_or(Object::Null) }
        Expr::Call { function, args } => {
            let mut eval_args = Vec::new(); for arg in args { eval_args.push(eval_expression(arg, env)); }
            match function.as_str() {
                "cmd" => { if let Some(Object::StringObj(command)) = eval_args.get(0) { if let Ok(output) = std::process::Command::new("sh").arg("-c").arg(command).output() { return Object::StringObj(String::from_utf8_lossy(&output.stdout).to_string()); } } Object::Null },
                "read" => { if let Some(Object::StringObj(path)) = eval_args.get(0) { if let Ok(content) = std::fs::read_to_string(path) { return Object::StringObj(content); } } Object::Null },
                "write" => { if let (Some(Object::StringObj(path)), Some(Object::StringObj(content))) = (eval_args.get(0), eval_args.get(1)) { let _ = std::fs::write(path, content); } Object::Null },
                "len" => { if let Some(Object::StringObj(s)) = eval_args.get(0) { return Object::Number(s.len() as f64); } if let Some(Object::Array(arr)) = eval_args.get(0) { return Object::Number(arr.len() as f64); } Object::Null },
                "push" => { if let (Some(Object::Array(mut arr)), Some(val)) = (eval_args.get(0).cloned(), eval_args.get(1).cloned()) { arr.push(val); return Object::Array(arr); } Object::Null },
                "rand" => { let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(); return Object::Number((t % 100) as f64); }
                _ => Object::Null
            }
        }
        Expr::BinaryOp { left, operator, right } => {
            let left_val = eval_expression(left, env); let right_val = eval_expression(right, env);
            if let (Object::Number(l), Object::Number(r)) = (&left_val, &right_val) { match operator.as_str() { "+" => Object::Number(l + r), "-" => Object::Number(l - r), "*" => Object::Number(l * r), "/" => if *r != 0.0 { Object::Number(l / r) } else { Object::Null }, "<" => Object::Boolean(l < r), ">" => Object::Boolean(l > r), "==" => Object::Boolean(l == r), "!=" => Object::Boolean(l != r), "<=" => Object::Boolean(l <= r), ">=" => Object::Boolean(l >= r), _ => Object::Null, } } else if let (Object::StringObj(l), Object::StringObj(r)) = (&left_val, &right_val) { if operator == "+" { Object::StringObj(format!("{}{}", l, r)) } else { Object::Null } } else { Object::Null }
        }
    }
}
