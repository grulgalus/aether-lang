use std::collections::HashMap;
use crate::ast::{Program, Stmt, Expr};
use crate::lexer::Lexer;
use crate::parser::Parser;

// PŘIDÁNO: Typ Error(String)!
#[derive(Debug, Clone, PartialEq)]
pub enum Object { Number(f64), StringObj(String), Boolean(bool), Array(Vec<Object>), Dict(HashMap<String, Object>), Null, Error(String) }

impl Object {
    pub fn to_string_val(&self) -> String {
        match self {
            Object::Number(n) => n.to_string(), Object::StringObj(s) => s.clone(), Object::Boolean(b) => b.to_string(),
            Object::Array(arr) => { let strings: Vec<String> = arr.iter().map(|x| match x { Object::StringObj(s) => format!("\"{}\"", s), _ => x.to_string_val() }).collect(); format!("[{}]", strings.join(", ")) },
            Object::Dict(d) => { let pairs: Vec<String> = d.iter().map(|(k, v)| format!("{}: {}", k, v.to_string_val())).collect(); format!("{{{}}}", pairs.join(", ")) },
            Object::Null => "null".to_string(),
            Object::Error(e) => format!("🛑 CHYBA BĚHU: {}", e), // Pokud chceš printnout error (což by se nemělo stát, protože ho zachytíme dřív)
        }
    }
}

pub struct Environment { pub store: HashMap<String, Object>, pub verbose: bool, pub output: Vec<String> }
impl Environment {
    pub fn new(verbose: bool) -> Self { Environment { store: HashMap::new(), verbose, output: Vec::new() } }
    pub fn set(&mut self, name: String, val: Object) { self.store.insert(name, val); }
    pub fn get(&self, name: &str) -> Option<Object> { self.store.get(name).cloned() }
}

pub fn eval_program(program: &Program, env: &mut Environment) -> Object { 
    let res = eval_block(&program.statements, env); 
    // Pokud na konci zjistíme, že to byl ERROR, napíšeme to jasně do výstupu!
    if let Object::Error(e) = &res {
        let msg = format!("🛑 KRITICKÁ CHYBA: {}", e);
        println!("{}", msg);
        env.output.push(msg);
    }
    res
}

fn eval_block(statements: &[Stmt], env: &mut Environment) -> Object { 
    let mut result = Object::Null; 
    for stmt in statements { 
        result = eval_statement(stmt, env); 
        // ERROR BUBBLING: Pokud nějaký řádek vyhodí chybu, OKAMŽITĚ přerušíme celý blok!
        if let Object::Error(_) = result { return result; } 
        if let Stmt::Return { .. } = stmt { break; } 
    } 
    result 
}

fn eval_statement(stmt: &Stmt, env: &mut Environment) -> Object {
    match stmt {
        Stmt::Import(path) => { if let Ok(content) = std::fs::read_to_string(path) { let lexer = Lexer::new(&content); let mut parser = Parser::new(lexer); let program = parser.parse_program(); eval_block(&program.statements, env); } Object::Null }
        Stmt::Let { name, value } | Stmt::Assign { name, value } => { 
            let val = eval_expression(value, env); 
            if let Object::Error(_) = val { return val; } // Zastav při chybě napravo
            env.set(name.clone(), val); 
            Object::Null 
        }
        Stmt::Expression(expr) => {
            let val = eval_expression(expr, env);
            if let Object::Error(_) = val { return val; }
            Object::Null
        },
        Stmt::Return { value } => eval_expression(value, env),
        Stmt::Print { value } => { 
            let val = eval_expression(value, env);
            if let Object::Error(_) = val { return val; }
            let text = val.to_string_val(); 
            println!("{}", text); 
            env.output.push(text); 
            Object::Null 
        }
        Stmt::Actor { methods, .. } => eval_block(methods, env),
        Stmt::Function { body, .. } => eval_block(body, env),
        Stmt::If { condition, consequence, alternative } => { 
            let cond = eval_expression(condition, env); 
            if let Object::Error(_) = cond { return cond; }
            let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true }; 
            if is_truthy { eval_block(consequence, env) } else if let Some(alt) = alternative { eval_block(alt, env) } else { Object::Null } 
        }
        Stmt::While { condition, body } => { 
            let mut res = Object::Null; 
            loop { 
                let cond = eval_expression(condition, env); 
                if let Object::Error(_) = cond { return cond; }
                let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true }; 
                if !is_truthy { break; } 
                res = eval_block(body, env); 
                if let Object::Error(_) = res { return res; }
            } 
            res 
        }
        Stmt::For { variable, iterable, body } => { 
            let iter_val = eval_expression(iterable, env); 
            if let Object::Error(_) = iter_val { return iter_val; }
            let mut res = Object::Null; 
            if let Object::Array(arr) = iter_val { 
                for item in arr { 
                    env.set(variable.clone(), item); 
                    res = eval_block(body, env); 
                    if let Object::Error(_) = res { return res; }
                } 
            } else { return Object::Error("Cyklus 'for' lze použít POUZE na pole!".to_string()); }
            res 
        }
    }
}

fn eval_expression(expr: &Expr, env: &Environment) -> Object {
    match expr {
        Expr::Number(s) => Object::Number(s.parse().unwrap_or(0.0)), Expr::StringLit(s) => Object::StringObj(s.clone()), Expr::Boolean(b) => Object::Boolean(*b),
        Expr::Array(elements) => { 
            let mut evaluated = Vec::new(); 
            for el in elements { let e = eval_expression(el, env); if let Object::Error(_) = e { return e; } evaluated.push(e); } 
            Object::Array(evaluated) 
        }
        Expr::Dict(pairs) => { 
            let mut map = HashMap::new(); 
            for (k, v) in pairs { let val = eval_expression(v, env); if let Object::Error(_) = val { return val; } map.insert(k.clone(), val); } 
            Object::Dict(map) 
        }
        Expr::Index { left, index } => { 
            let left_val = eval_expression(left, env); if let Object::Error(_) = left_val { return left_val; }
            let index_val = eval_expression(index, env); if let Object::Error(_) = index_val { return index_val; }
            
            if let (Object::Array(arr), Object::Number(i)) = (&left_val, &index_val) { 
                let idx = *i as usize; 
                if idx < arr.len() { return arr[idx].clone(); } else { return Object::Error(format!("Index {} je mimo velikost pole (max: {})", idx, arr.len() - 1)); }
            } 
            if let (Object::Dict(map), Object::StringObj(k)) = (&left_val, &index_val) { 
                if let Some(val) = map.get(k) { return val.clone(); } else { return Object::Error(format!("Klíč '{}' ve slovníku neexistuje!", k)); }
            } 
            Object::Error("Indexovat ([]) lze pouze pole a slovníky!".to_string()) 
        }
        Expr::Identifier(s) => { 
            if s == "input" { let mut line = String::new(); std::io::stdin().read_line(&mut line).unwrap_or(0); return Object::StringObj(line.trim().to_string()); } 
            if let Some(val) = env.get(s) { return val; }
            // TADY TO JE! ŽÁDNÝ NULL! CHYBA!
            Object::Error(format!("Neznámá proměnná '{}'. Nezapomněl jsi 'let'?", s))
        }
        Expr::Call { function, args } => {
            let mut eval_args = Vec::new(); 
            for arg in args { let e = eval_expression(arg, env); if let Object::Error(_) = e { return e; } eval_args.push(e); }
            match function.as_str() {
                "env" => { if let Some(Object::StringObj(k)) = eval_args.get(0) { return Object::StringObj(std::env::var(k).unwrap_or_else(|_| "".to_string())); } Object::Error("env() vyžaduje název (text)".to_string()) },
                "load_env" => { if let Some(Object::StringObj(path)) = eval_args.get(0) { let mut map = HashMap::new(); if let Ok(content) = std::fs::read_to_string(path) { for line in content.lines() { let l = line.trim(); if l.is_empty() || l.starts_with('#') { continue; } if let Some((k, v)) = l.split_once('=') { let val = v.trim().trim_matches('"').trim_matches('\''); map.insert(k.trim().to_string(), Object::StringObj(val.to_string())); } } } return Object::Dict(map); } Object::Error("load_env() vyžaduje platnou cestu".to_string()) },
                "parse_json" => { if let Some(Object::StringObj(content)) = eval_args.get(0) { let lexer = Lexer::new(&content); let mut parser = Parser::new(lexer); let program = parser.parse_program(); if let Some(Stmt::Expression(expr)) = program.statements.first() { return eval_expression(expr, env); } } Object::Error("Chyba při parsování JSON".to_string()) },
                "read" => { if let Some(Object::StringObj(path)) = eval_args.get(0) { if let Ok(content) = std::fs::read_to_string(path) { return Object::StringObj(content); } else { return Object::Error(format!("Soubor '{}' nebylo možné přečíst!", path)); } } Object::Error("read() vyžaduje cestu k souboru".to_string()) },
                "type" => { if let Some(arg) = eval_args.get(0) { return Object::StringObj(match arg { Object::Number(_) => "number", Object::StringObj(_) => "string", Object::Boolean(_) => "boolean", Object::Array(_) => "array", Object::Dict(_) => "dict", Object::Null => "null", Object::Error(_) => "error" }.to_string()); } Object::Error("type() vyžaduje 1 argument".to_string()) },
                "time" => { let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(); return Object::Number(t as f64); },
                "keys" => { if let Some(Object::Dict(map)) = eval_args.get(0) { return Object::Array(map.keys().map(|k| Object::StringObj(k.clone())).collect()); } Object::Error("keys() vyžaduje slovník".to_string()) },
                "len" => { if let Some(Object::StringObj(s)) = eval_args.get(0) { return Object::Number(s.len() as f64); } if let Some(Object::Array(arr)) = eval_args.get(0) { return Object::Number(arr.len() as f64); } Object::Error("len() lze použít jen na text nebo pole".to_string()) },
                "push" => { if let (Some(Object::Array(mut arr)), Some(val)) = (eval_args.get(0).cloned(), eval_args.get(1).cloned()) { arr.push(val); return Object::Array(arr); } Object::Error("push() vyžaduje (pole, hodnotu)".to_string()) },
                "rand" => { let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(); return Object::Number((t % 100) as f64); },
                "upper" => { if let Some(Object::StringObj(s)) = eval_args.get(0) { return Object::StringObj(s.to_uppercase()); } Object::Error("upper() vyžaduje text".to_string()) },
                "lower" => { if let Some(Object::StringObj(s)) = eval_args.get(0) { return Object::StringObj(s.to_lowercase()); } Object::Error("lower() vyžaduje text".to_string()) },
                "split" => { if let (Some(Object::StringObj(s)), Some(Object::StringObj(d))) = (eval_args.get(0), eval_args.get(1)) { return Object::Array(s.split(d.as_str()).map(|x| Object::StringObj(x.to_string())).collect()); } Object::Error("split() vyžaduje (text, oddělovač)".to_string()) },
                "str" => { if let Some(obj) = eval_args.get(0) { return Object::StringObj(obj.to_string_val()); } Object::Null },
                "int" => { if let Some(obj) = eval_args.get(0) { match obj { Object::StringObj(s) => return Object::Number(s.parse().unwrap_or(0.0)), Object::Number(n) => return Object::Number(*n), Object::Boolean(b) => return Object::Number(if *b { 1.0 } else { 0.0 }), _ => return Object::Error("Tento typ nelze převést na číslo".to_string()) } } Object::Null },
                _ => Object::Error(format!("Funkce '{}()' neexistuje!", function))
            }
        }
        Expr::BinaryOp { left, operator, right } => {
            let left_val = eval_expression(left, env); if let Object::Error(_) = left_val { return left_val; }
            let right_val = eval_expression(right, env); if let Object::Error(_) = right_val { return right_val; }
            
            if operator == "&&" {
                let is_l = match &left_val { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, Object::Null => false, _ => true };
                let is_r = match &right_val { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, Object::Null => false, _ => true };
                return Object::Boolean(is_l && is_r);
            }
            if operator == "||" {
                let is_l = match &left_val { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, Object::Null => false, _ => true };
                let is_r = match &right_val { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, Object::Null => false, _ => true };
                return Object::Boolean(is_l || is_r);
            }
            if let (Object::Number(l), Object::Number(r)) = (&left_val, &right_val) { match operator.as_str() { "+" => Object::Number(l + r), "-" => Object::Number(l - r), "*" => Object::Number(l * r), "/" => if *r != 0.0 { Object::Number(l / r) } else { Object::Error("Dělení nulou je zakázáno!".to_string()) }, "<" => Object::Boolean(l < r), ">" => Object::Boolean(l > r), "==" => Object::Boolean(l == r), "!=" => Object::Boolean(l != r), "<=" => Object::Boolean(l <= r), ">=" => Object::Boolean(l >= r), _ => Object::Error(format!("Neznámý operátor: {}", operator)), } } 
            else if let (Object::StringObj(l), Object::StringObj(r)) = (&left_val, &right_val) { if operator == "+" { Object::StringObj(format!("{}{}", l, r)) } else if operator == "==" { Object::Boolean(l == r) } else if operator == "!=" { Object::Boolean(l != r) } else { Object::Error(format!("Operátor '{}' nelze použít na text", operator)) } } 
            else { Object::Error(format!("Špatné datové typy. Nelze použít '{}' na tyto hodnoty.", operator)) }
        }
    }
}
