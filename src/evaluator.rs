use std::collections::HashMap;
use std::net::TcpListener;
use std::io::{Read, Write};
use std::process::Command;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::ast::{Program, Stmt, Expr};
use crate::lexer::Lexer;
use crate::parser::Parser;

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
        
        Stmt::Import(path) => {
            if let Ok(content) = fs::read_to_string(path) {
                let lexer = Lexer::new(&content); let mut parser = Parser::new(lexer); let prog = parser.parse_program();
                eval_block(&prog.statements, env); Object::Null
            } else { Object::Error(format!("Modul '{}' nenalezen!", path)) }
        },
        Stmt::Function { .. } | Stmt::Actor { .. } => Object::Null,
    }
}

// 🔧 OPRAVA TADY: Změněno z `&Environment` na `&mut Environment`!
fn eval_expression(expr: &Expr, env: &mut Environment) -> Object {
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
                "len" => if let Some(Object::StringObj(s)) = eval_args.get(0) { Object::Number(s.len() as f64) } else if let Some(Object::Array(arr)) = eval_args.get(0) { Object::Number(arr.len() as f64) } else { Object::Error("len() bere text/pole".into()) },
                "str" => if let Some(obj) = eval_args.get(0) { Object::StringObj(obj.to_string_val()) } else { Object::Null },
                "int" => if let Some(Object::StringObj(s)) = eval_args.get(0) { Object::Number(s.parse().unwrap_or(0.0)) } else { Object::Error("int() bere text".into()) },
                "push" => if let (Some(Object::Array(mut arr)), Some(val)) = (eval_args.get(0).cloned(), eval_args.get(1).cloned()) { arr.push(val); Object::Array(arr) } else { Object::Error("push() vyžaduje (pole, hodnota)".into()) },
                "pop" => if let Some(Object::Array(mut arr)) = eval_args.get(0).cloned() { let last = arr.pop().unwrap_or(Object::Null); Object::Array(arr) } else { Object::Error("pop() vyžaduje pole".into()) },
                
                "upper" => if let Some(Object::StringObj(s)) = eval_args.get(0) { Object::StringObj(s.to_uppercase()) } else { Object::Error("upper() vyžaduje text".into()) },
                "lower" => if let Some(Object::StringObj(s)) = eval_args.get(0) { Object::StringObj(s.to_lowercase()) } else { Object::Error("lower() vyžaduje text".into()) },
                "trim" => if let Some(Object::StringObj(s)) = eval_args.get(0) { Object::StringObj(s.trim().to_string()) } else { Object::Error("trim() vyžaduje text".into()) },
                "replace" => if let (Some(Object::StringObj(s)), Some(Object::StringObj(o)), Some(Object::StringObj(n))) = (eval_args.get(0), eval_args.get(1), eval_args.get(2)) { Object::StringObj(s.replace(o, n)) } else { Object::Error("replace() vyžaduje (text, najit, nahradit)".into()) },
                "contains" => if let (Some(Object::StringObj(s)), Some(Object::StringObj(f))) = (eval_args.get(0), eval_args.get(1)) { Object::Boolean(s.contains(f)) } else { Object::Error("contains() vyžaduje (text, hledat)".into()) },
                "split" => if let (Some(Object::StringObj(s)), Some(Object::StringObj(d))) = (eval_args.get(0), eval_args.get(1)) { let arr = s.split(d).map(|x| Object::StringObj(x.to_string())).collect(); Object::Array(arr) } else { Object::Error("split() vyžaduje (text, oddelovac)".into()) },

                "sqrt" => if let Some(Object::Number(n)) = eval_args.get(0) { Object::Number(n.sqrt()) } else { Object::Error("sqrt() vyžaduje číslo".into()) },
                "round" => if let Some(Object::Number(n)) = eval_args.get(0) { Object::Number(n.round()) } else { Object::Error("round() vyžaduje číslo".into()) },
                "abs" => if let Some(Object::Number(n)) = eval_args.get(0) { Object::Number(n.abs()) } else { Object::Error("abs() vyžaduje číslo".into()) },
                "time" => { let ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(); Object::Number(ms as f64) },
                "rand" => { let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos(); Object::Number((nanos % 100) as f64) },
                
                "read" => if let Some(Object::StringObj(p)) = eval_args.get(0) { match fs::read_to_string(p) { Ok(c) => Object::StringObj(c), Err(_) => Object::Error(format!("Soubor '{}' nenalezen!", p)) } } else { Object::Error("read() vyžaduje cestu".into()) },
                "write" => if let (Some(Object::StringObj(p)), Some(Object::StringObj(c))) = (eval_args.get(0), eval_args.get(1)) { match fs::write(p, c) { Ok(_) => Object::Boolean(true), Err(_) => Object::Error("Chyba zápisu".into()) } } else { Object::Error("write() vyžaduje (cestu, text)".into()) },
                "sleep" => if let Some(Object::Number(ms)) = eval_args.get(0) { std::thread::sleep(std::time::Duration::from_millis(*ms as u64)); Object::Null } else { Object::Error("sleep() vyžaduje ms".into()) },
                "clear" => { print!("\x1B[2J\x1B[1;1H"); let _ = std::io::stdout().flush(); Object::Null },
                "cmd" => if let Some(Object::StringObj(c)) = eval_args.get(0) { if let Ok(out) = Command::new("sh").arg("-c").arg(c).output() { Object::StringObj(String::from_utf8_lossy(&out.stdout).trim().to_string()) } else { Object::Error("Příkaz selhal".into()) } } else { Object::Error("cmd() vyžaduje příkaz".into()) },
                
                "fetch" => if let Some(Object::StringObj(url)) = eval_args.get(0) { if let Ok(out) = Command::new("curl").arg("-s").arg(url).output() { Object::StringObj(String::from_utf8_lossy(&out.stdout).trim().to_string()) } else { Object::Error("Fetch selhal".into()) } } else { Object::Error("fetch() vyžaduje url".into()) },
                "serve" => if let (Some(Object::Number(port)), Some(Object::StringObj(html))) = (eval_args.get(0), eval_args.get(1)) { let addr = format!("127.0.0.1:{}", *port as u16); println!("🌐 Aether Server: http://{}", addr); if let Ok(listener) = TcpListener::bind(&addr) { for stream in listener.incoming() { if let Ok(mut stream) = stream { let mut b = [0; 512]; let _ = stream.read(&mut b); let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", html); let _ = stream.write_all(resp.as_bytes()); } } Object::Null } else { Object::Error("Port je obsazen!".into()) } } else { Object::Error("serve(port, html)".into()) },
                "discord" => if let (Some(Object::StringObj(url)), Some(Object::StringObj(msg))) = (eval_args.get(0), eval_args.get(1)) { let json = format!(r#"{{"content": "{}"}}"#, msg); if Command::new("curl").arg("-s").arg("-H").arg("Content-Type: application/json").arg("-d").arg(&json).arg(url).status().is_ok() { Object::Boolean(true) } else { Object::Error("Discord selhal".into()) } } else { Object::Error("discord(url, msg)".into()) },
                
                // Nyní exec() hladce použije náš mutabilní paměťový blok (env)!
                "exec" => if let Some(Object::StringObj(code)) = eval_args.get(0) { let lexer = Lexer::new(&code); let mut parser = Parser::new(lexer); let prog = parser.parse_program(); eval_block(&prog.statements, env); Object::Null } else { Object::Error("exec() vyžaduje textový kód".into()) },

                _ => Object::Error(format!("Neznámá funkce '{}()'", function))
            }
        }
        Expr::BinaryOp { left, operator, right } => {
            let l = eval_expression(left, env); if let Object::Error(_) = l { return l; }
            let r = eval_expression(right, env); if let Object::Error(_) = r { return r; }
            if operator == "&&" { let is_l = match &l { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, _ => true }; let is_r = match &r { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, _ => true }; return Object::Boolean(is_l && is_r); }
            if operator == "||" { let is_l = match &l { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, _ => true }; let is_r = match &r { Object::Boolean(b) => *b, Object::Number(n) => *n != 0.0, _ => true }; return Object::Boolean(is_l || is_r); }
            if let (Object::Number(v1), Object::Number(v2)) = (&l, &r) { match operator.as_str() { "+" => Object::Number(v1 + v2), "-" => Object::Number(v1 - v2), "*" => Object::Number(v1 * v2), "/" => if *v2 != 0.0 { Object::Number(v1 / v2) } else { Object::Error("Dělení nulou".into()) }, "<" => Object::Boolean(v1 < v2), ">" => Object::Boolean(v1 > v2), "==" => Object::Boolean(v1 == v2), "!=" => Object::Boolean(v1 != v2), "<=" => Object::Boolean(v1 <= v2), ">=" => Object::Boolean(v1 >= v2), _ => Object::Error("Neznámý operátor".into()) } } else if let (Object::StringObj(s1), Object::StringObj(s2)) = (&l, &r) { if operator == "+" { Object::StringObj(format!("{}{}", s1, s2)) } else if operator == "==" { Object::Boolean(s1 == s2) } else if operator == "!=" { Object::Boolean(s1 != s2) } else { Object::Error("Na text nelze použít tento operátor".into()) } } else { Object::Error("Matematika vyžaduje čísla nebo text".into()) }
        }
    }
}
