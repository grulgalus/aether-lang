use std::collections::HashMap;
use crate::ast::{Program, Stmt, Expr};

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Number(f64),
    StringObj(String),
    Null,
}

pub struct Environment {
    store: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self { Environment { store: HashMap::new() } }
    pub fn set(&mut self, name: String, val: Object) { self.store.insert(name, val); }
    pub fn get(&self, name: &str) -> Option<Object> { self.store.get(name).cloned() }
}

pub fn eval_program(program: &Program, env: &mut Environment) -> Object {
    let mut result = Object::Null;
    for stmt in &program.statements { result = eval_statement(stmt, env); }
    result
}

fn eval_statement(stmt: &Stmt, env: &mut Environment) -> Object {
    match stmt {
        Stmt::Let { name, value } => {
            let val = eval_expression(value, env);
            env.set(name.clone(), val.clone());
            Object::Null
        }
        Stmt::Return { value } => eval_expression(value, env),
        Stmt::Print { value } => {
            let val = eval_expression(value, env);
            match val {
                Object::Number(n) => println!("{}", n),
                Object::StringObj(s) => println!("{}", s),
                Object::Null => println!("null"),
            }
            Object::Null
        }
        Stmt::Actor { name: _, methods } => {
            let mut res = Object::Null;
            for method in methods { res = eval_statement(method, env); }
            res
        }
        Stmt::Function { name: _, body } => {
            let mut res = Object::Null;
            for b_stmt in body {
                res = eval_statement(b_stmt, env);
                if let Stmt::Return { .. } = b_stmt { break; }
            }
            res
        }
    }
}

fn eval_expression(expr: &Expr, env: &Environment) -> Object {
    match expr {
        Expr::Number(s) => Object::Number(s.parse().unwrap_or(0.0)),
        Expr::StringLit(s) => Object::StringObj(s.clone()),
        Expr::Identifier(s) => env.get(s).unwrap_or(Object::Null),
        
        // Zde řešíme výpočty!
        Expr::BinaryOp { left, operator, right } => {
            let left_val = eval_expression(left, env);
            let right_val = eval_expression(right, env);
            
            // Pokud jsou obě strany čísla:
            if let (Object::Number(l), Object::Number(r)) = (&left_val, &right_val) {
                match operator.as_str() {
                    "+" => Object::Number(l + r),
                    "-" => Object::Number(l - r),
                    "*" => Object::Number(l * r),
                    "/" => if *r != 0.0 { Object::Number(l / r) } else { Object::Null },
                    _ => Object::Null,
                }
            } 
            // Pokud jsou obě strany text (např. "Ahoj " + "světe"):
            else if let (Object::StringObj(l), Object::StringObj(r)) = (&left_val, &right_val) {
                if operator == "+" {
                    Object::StringObj(format!("{}{}", l, r))
                } else {
                    Object::Null
                }
            } else {
                Object::Null
            }
        }
    }
}
