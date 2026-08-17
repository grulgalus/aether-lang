use std::collections::HashMap;
use std::net::TcpListener;
use std::io::{Read, Write};
use std::process::Command; // PŘIDÁNO PRO TERMINÁL A DISCORD
use crate::ast::{Program, Stmt, Expr};

#[derive(Debug, Clone, PartialEq)]
pub enum Object { Number(f64), StringObj(String), Boolean(bool), Array(Vec<Object>), Dict(HashMap<String, Object>), Null, Error(String) }

impl Object {
    pub fn to_string_val(&self) -> String {
        match self {
            Object::Number(n) => n.to_string(), Object::StringObj(s) => s.clone(), Object::Boolean(b) => b.to_string(),
            Object::Array(arr) => { let strings: Vec<String> = arr.iter().map(|x| match x { Object::StringObj(s) => format!("\"{}\"", s), _ => x.to_string_val() }).collect(); format!("[{}]", strings.join(", ")) },
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

#[allow(dead_code)] fn check_program(_program: &Program) -> Result<(), String> { Ok(()) }
#[allow(dead_code)] fn optimize_expr(expr: Expr) -> Expr { expr }

pub fn eval_program(program: &Program, env: &mut Environment) -> Object {
    if let Err(e) = check_program(program) { return Object::Error(format!("Chyba analýzy: {}", e)); }
    let mut result = Object::Null;
    for stmt in &program.statements { result = eval_statement(stmt, env); if let Object::Error(e) = result { return Object::Error(e); } }
    result
}

fn eval_block(stmts: &[Stmt], env: &mut Environment) -> Object {
    let mut res = Object::Null;
    for s in stmts { res = eval_statement(s, env); if let Object::Error(_) = res { return res; } }
    res
}

fn eval_statement(stmt: &Stmt, env: &mut Environment) -> Object {
    match stmt {
        Stmt::Let { name, value } | Stmt::Assign { name, value } => { let val = eval_expression(value, env); if let Object::Error(_) = val { return val; } env.set(name.clone(), val); Object::Null }
        Stmt::Print { value } => { let val = eval_expression(value, env); if let Object::Error(_) = val { return val; } let text = val.to_string_val(); env.output.push(text.clone()); println!("{}", text); Object::Null }
        Stmt::If { condition, consequence, alternative } => { let cond = eval_expression(condition, env); if let Object::Error(_) = cond { return cond; } let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true }; if is_truthy { eval_block(consequence, env) } else if let Some(alt) = alternative { eval_block(alt, env) } else { Object::Null } }
        Stmt::While { condition, body } => { let mut res = Object::Null; loop { let cond = eval_expression(condition, env); if let Object::Error(_) = cond { return cond; } let is_truthy = match cond { Object::Boolean(b) => b, Object::Number(n) => n != 0.0, Object::Null => false, _ => true }; if !is_truthy { break; } res = eval_block(body, env); if let Object::Error(_) = res { return res; } } res }
        Stmt::For { variable, iterable, body } => { let iter_val = eval_expression(iterable, env); if let Object::Array(arr) = iter_val { for item in arr { env.set(variable.clone(), item); let res = eval_block(body, env); if let Object::Error(_) = res { return res; } } Object::Null } else { Object::Error("For vyžaduje pole".into()) } }
        Stmt::Expression(expr) => eval_expression(expr, env),
        Stmt::Return { value } => eval_expression(value, env),
        Stmt::Function { .. } | Stmt::Actor { .. } | Stmt::Import(_) => Object::Null,
    }
}

fn eval_expression(expr: &Expr, env: &Environment) -> Object {
    match expr {
        Expr::Number(n) => Object::Number(n.parse().unwrap_or(0.0)), Expr::StringLit(s) => Object::StringObj(s.clone()), Expr::Boolean(b) => Object::Boolean(*b),
        Expr::Identifier(s) => env.get(s).unwrap_or(Object::Error(format!("Neznámá proměnná '{}'", s))),
        Expr::Array(els) => { let mut arr = Vec::new(); for el in els { let e = eval_expression(el, env); if let Object::Error(_) = e { return e; } arr.push(e); } Object::Array(arr) }
        Expr::Dict(pairs) => { let mut map = HashMap::new(); for (k, v) in pairs { let val = eval_expression(v, env); if let Object::Error(_) = val { return val; } map.insert(k.clone(), val); } Object::Dict(map) }
        Expr::Index { left, index } => { let l = eval_expression(left, env); if let Object::Error(_) = l { return l; } let i = eval_expression(index, env); if let Object::Error(_) = i { return i; } if let (Object::Array(arr), Object::Number(idx)) = (&l, &i) { let index_usize = *idx as usize; if index_usize < arr.len() { return arr[index_usize].clone(); } else { return Object::Error("Index je mimo pole".into()); } } if let (Object::Dict(map), Object::StringObj(k)) = (&l, &i) { if let Some(val) = map.get(k) { return val.clone(); } else { return Object::Error("Klíč neexistuje".into()); } } Object::Error("Lze indexovat jen pole a dict".into()) }
        Expr::Call { function, args } => {
            let mut eval_args = Vec::new(); 
            for arg in args { let e = eval_expression(arg, env); if let Object::Error(_) = e { return e; } eval_args.push(e); }
            match function.as_str() {
                "len" => { if let Some(Object::StringObj(s)) = eval_args.get(0) { return Object::Number(s.len() as f64); } if let Some(Object::Array(arr)) = eval_args.get(0) { return Object::Number(arr.len() as f64); } Object::Error("len() bere text/pole".into()) },
                "push" => { if let (Some(Object::Array(mut arr)), Some(val)) = (eval_args.get(0).cloned(), eval_args.get(1).cloned()) { arr.push(val); return Object::Array(arr); } Object::Error("push() vyžaduje (pole, hodnota)".into()) },
                "upper" => { if let Some(Object::StringObj(s)) = eval_args.get(0) { return Object::StringObj(s.to_uppercase()); } Object::Error("upper() vyžaduje text".to_string()) },
                "lower" => { if let Some(Object::StringObj(s)) = eval_args.get(0) { return Object::StringObj(s.to_lowercase()); } Object::Error("lower() vyžaduje text".to_string()) },
                "str" => { if let Some(obj) = eval_args.get(0) { return Object::StringObj(obj.to_string_val()); } Object::Null },
                "serve" => {
                    if let (Some(Object::Number(port)), Some(Object::StringObj(html))) = (eval_args.get(0), eval_args.get(1)) {
                        let addr = format!("127.0.0.1:{}", *port as u16); println!("🌐 Aether Server běží na http://{}", addr);
                        if let Ok(listener) = TcpListener::bind(&addr) { for stream in listener.incoming() { if let Ok(mut stream) = stream { let mut buffer = [0; 512]; let _ = stream.read(&mut buffer); let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}", html); let _ = stream.write_all(response.as_bytes()); } } } else { return Object::Error(format!("Port {} je už obsazený!", port)); }
                        return Object::Null;
                    } Object::Error("serve() vyžaduje (číslo_portu, text_stránky)".into())
                },
                
                // 🚀 NOVINKA: TERMINÁLOVÉ PŘÍKAZY
                "cmd" => {
                    if let Some(Object::StringObj(c)) = eval_args.get(0) {
                        if let Ok(output) = Command::new("sh").arg("-c").arg(c).output() {
                            return Object::StringObj(String::from_utf8_lossy(&output.stdout).to_string().trim().to_string());
                        } return Object::Error("Příkaz selhal".into());
                    } Object::Error("cmd() vyžaduje textový příkaz".into())
                },
                
                // 🤖 NOVINKA: DISCORD API WEBHOOK BOT
                "discord" => {
                    if let (Some(Object::StringObj(url)), Some(Object::StringObj(msg))) = (eval_args.get(0), eval_args.get(1)) {
                        // Vytvoříme JSON pro Discord
                        let json = format!(r#"{{"content": "{}"}}"#, msg);
                        // Pošleme ho potichu přes systémový curl
                        let status = Command::new("curl").arg("-s").arg("-H").arg("Content-Type: application/json").arg("-d").arg(&json).arg(url).status();
                        if status.is_ok() { return Object::Boolean(true); } else { return Object::Error("Discord API selhalo (máš v Termuxu nainstalovaný 'curl'?)".into()); }
                    } Object::Error("discord() vyžaduje (webhook_url, zprava)".into())
                },
                _ => Object::Error(format!("Neznámá funkce '{}'", function))
            }
        }
        Expr::BinaryOp { left, operator, right } => {
            let l = eval_expression(left, env); if let Object::Error(_) = l { return l; }
            let r = eval_expression(right, env); if let Object::Error(_) = r { return r; }
            if operator == "&&" { let is_l = match &l { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, _ => true }; let is_r = match &r { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, _ => true }; return Object::Boolean(is_l && is_r); }
            if operator == "||" { let is_l = match &l { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, _ => true }; let is_r = match &r { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, _ => true }; return Object::Boolean(is_l || is_r); }
            if let (Object::Number(v1), Object::Number(v2)) = (&l, &r) { match operator.as_str() { "+" => Object::Number(v1 + v2), "-" => Object::Number(v1 - v2), "*" => Object::Number(v1 * v2), "/" => if *v2 != 0.0 { Object::Number(v1 / v2) } else { Object::Error("Dělení nulou".into()) }, "<" => Object::Boolean(v1 < v2), ">" => Object::Boolean(v1 > v2), "==" => Object::Boolean(v1 == v2), "!=" => Object::Boolean(v1 != v2), "<=" => Object::Boolean(v1 <= v2), ">=" => Object::Boolean(v1 >= v2), _ => Object::Error("Neznámý operátor".into()) } } else if let (Object::StringObj(s1), Object::StringObj(s2)) = (&l, &r) { if operator == "+" { Object::StringObj(format!("{}{}", s1, s2)) } else if operator == "==" { Object::Boolean(s1 == s2) } else if operator == "!=" { Object::Boolean(s1 != s2) } else { Object::Error("Na text nelze použít tento operátor".into()) } } else { Object::Error("Matematika vyžaduje čísla nebo spojování textu".into()) }
        }
    }
}
