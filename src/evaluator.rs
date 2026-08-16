use std::collections::HashMap;
use crate::ast::{Program, Stmt, Expr};

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Number(f64),
    StringObj(String),
    Null,
}

// Paměť našeho jazyka (ukládá hodnoty proměnných)
pub struct Environment {
    store: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { store: HashMap::new() }
    }
    pub fn set(&mut self, name: String, val: Object) {
        self.store.insert(name, val);
    }
    pub fn get(&self, name: &str) -> Option<Object> {
        self.store.get(name).cloned()
    }
}

// Spuštění celého programu
pub fn eval_program(program: &Program, env: &mut Environment) -> Object {
    let mut result = Object::Null;
    for stmt in &program.statements {
        result = eval_statement(stmt, env);
    }
    result
}

// Vykonání jednotlivých příkazů
fn eval_statement(stmt: &Stmt, env: &mut Environment) -> Object {
    match stmt {
        Stmt::Let { name, value } => {
            let val = eval_expression(value, env);
            env.set(name.clone(), val.clone());
            Object::Null
        }
        Stmt::Return { value } => {
            eval_expression(value, env)
        }
        Stmt::Actor { name: _, methods } => {
            // Pro jednoduchost teď necháme actora rovnou spustit všechny své metody
            let mut res = Object::Null;
            for method in methods {
                res = eval_statement(method, env);
            }
            res
        }
        Stmt::Function { name: _, body } => {
            // Vykonání těla funkce
            let mut res = Object::Null;
            for b_stmt in body {
                res = eval_statement(b_stmt, env);
                // Pokud narazíme na return, vrátíme hodnotu
                if let Stmt::Return { .. } = b_stmt {
                    break;
                }
            }
            res
        }
    }
}

// Vyhodnocení výrazů (čísla, texty, proměnné)
fn eval_expression(expr: &Expr, env: &Environment) -> Object {
    match expr {
        Expr::Number(s) => Object::Number(s.parse().unwrap_or(0.0)),
        Expr::StringLit(s) => Object::StringObj(s.clone()),
        Expr::Identifier(s) => env.get(s).unwrap_or(Object::Null),
    }
}
