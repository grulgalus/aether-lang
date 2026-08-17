use std::collections::HashMap;
use crate::ast::{Program, Stmt, Expr};

#[derive(Debug, Clone, PartialEq)]
pub enum Object { Number(f64), StringObj(String), Boolean(bool), Null }

pub struct Environment { store: HashMap<String, Object> }

impl Environment {
    pub fn new() -> Self { Environment { store: HashMap::new() } }
    pub fn set(&mut self, name: String, val: Object) { self.store.insert(name, val); }
    pub fn get(&self, name: &str) -> Option<Object> { self.store.get(name).cloned() }
}

pub fn eval_program(program: &Program, env: &mut Environment) -> Object {
    eval_block(&program.statements, env)
}

fn eval_block(statements: &[Stmt], env: &mut Environment) -> Object {
    let mut result = Object::Null;
    for stmt in statements {
        result = eval_statement(stmt, env);
        if let Stmt::Return { .. } = stmt { break; }
    }
    result
}

fn eval_statement(stmt: &Stmt, env: &mut Environment) -> Object {
    match stmt {
        Stmt::Let { name, value } | Stmt::Assign { name, value } => {
            let val = eval_expression(value, env);
            env.set(name.clone(), val);
            Object::Null
        }
        Stmt::Return { value } => eval_expression(value, env),
        Stmt::Print { value } => {
            match eval_expression(value, env) {
                Object::Number(n) => println!("{}", n),
                Object::StringObj(s) => println!("{}", s),
                Object::Boolean(b) => println!("{}", b),
                Object::Null => println!("null"),
            }
            Object::Null
        }
        Stmt::Actor { methods, .. } => eval_block(methods, env),
        Stmt::Function { body, .. } => eval_block(body, env),
        Stmt::If { condition, consequence, alternative } => {
            let cond = eval_expression(condition, env);
            let is_truthy = match cond {
                Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true,
            };
            if is_truthy { eval_block(consequence, env) } else if let Some(alt) = alternative { eval_block(alt, env) } else { Object::Null }
        }
        Stmt::While { condition, body } => {
            let mut res = Object::Null;
            loop {
                let cond = eval_expression(condition, env);
                let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true };
                if !is_truthy { break; }
                res = eval_block(body, env);
            }
            res
        }
    }
}

fn eval_expression(expr: &Expr, env: &Environment) -> Object {
    match expr {
        Expr::Number(s) => Object::Number(s.parse().unwrap_or(0.0)),
        Expr::StringLit(s) => Object::StringObj(s.clone()),
        Expr::Boolean(b) => Object::Boolean(*b),
        Expr::Identifier(s) => env.get(s).unwrap_or(Object::Null),
        Expr::BinaryOp { left, operator, right } => {
            let left_val = eval_expression(left, env);
            let right_val = eval_expression(right, env);
            if let (Object::Number(l), Object::Number(r)) = (&left_val, &right_val) {
                match operator.as_str() {
                    "+" => Object::Number(l + r), "-" => Object::Number(l - r),
                    "*" => Object::Number(l * r), "/" => if *r != 0.0 { Object::Number(l / r) } else { Object::Null },
                    "<" => Object::Boolean(l < r), ">" => Object::Boolean(l > r),
                    "==" => Object::Boolean(l == r), "!=" => Object::Boolean(l != r),
                    "<=" => Object::Boolean(l <= r), ">=" => Object::Boolean(l >= r),
                    _ => Object::Null,
                }
            } else if let (Object::StringObj(l), Object::StringObj(r)) = (&left_val, &right_val) {
                if operator == "+" { Object::StringObj(format!("{}{}", l, r)) } else { Object::Null }
            } else { Object::Null }
        }
    }
}
