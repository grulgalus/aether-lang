use std::env;
use std::fs;
use std::time::Instant;
use std::process::Command;
use std::io::{Read, Write};
use std::net::TcpListener;

mod lexer; mod ast; mod parser; mod evaluator;

fn detect_editor() -> String { if let Ok(ed) = env::var("EDITOR") { if !ed.is_empty() { return ed; } } for ed in ["nano", "vim", "nvim", "vi", "emacs"].iter() { if let Ok(out) = Command::new("which").arg(ed).output() { if out.status.success() { return ed.to_string(); } } } "nano".to_string() }
struct Config { language: String, auto_open_broken: bool, auto_verbose: bool, editor: String }
impl Config { fn load() -> Self { let path = format!("{}/.aether_config", env::var("HOME").unwrap_or_else(|_| ".".to_string())); let mut c = Config { language: "en".to_string(), auto_open_broken: false, auto_verbose: false, editor: detect_editor() }; if let Ok(content) = fs::read_to_string(&path) { for l in content.lines() { let p: Vec<&str> = l.split('=').collect(); if p.len() == 2 { match p[0].trim() { "language-of-aether" => c.language = p[1].trim().to_string(), "auto-open-file-if-is-broken" => c.auto_open_broken = p[1].trim() == "on", "auto-stop-shut-up-compilator" => c.auto_verbose = p[1].trim() == "on", "default-editor-command" => c.editor = p[1].trim().to_string(), _ => {} } } } } else { let _ = fs::write(&path, format!("language-of-aether=en\nauto-open-file-if-is-broken=off\nauto-stop-shut-up-compilator=off\ndefault-editor-command={}\n", c.editor)); } c } }

fn serve_studio() {
    let listener = TcpListener::bind("127.0.0.1:8765").expect("Port zabrany!");
    println!("🎨 AETHER STUDIO BĚŽÍ na http://127.0.0.1:8765");
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buf = [0; 32768];
            if let Ok(br) = stream.read(&mut buf) {
                let req = String::from_utf8_lossy(&buf[..br]);
                if req.starts_with("GET /icon.png") {
                    if let Ok(img) = fs::read("res/aether_space_icon.png") { let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\n\r\n", img.len()).as_bytes()); let _ = stream.write_all(&img); } else { let _ = stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n"); }
                } else if req.starts_with("GET / ") {
                    let html = r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>Aether Studio</title><link rel="icon" type="image/png" href="/icon.png"><style>body{margin:0;background:#121212;color:#d4d4d4;font-family:monospace;display:flex;flex-direction:column;height:100vh}.header{background:#1e1e1e;padding:15px;display:flex;justify-content:space-between;align-items:center;border-bottom:2px solid #333}.header-left{display:flex;align-items:center;gap:15px}.logo{width:40px;height:40px;border-radius:8px}.header h1{margin:0;color:#4af626}button{background:#00bcd4;color:#121212;border:none;padding:10px 20px;font-size:16px;cursor:pointer;font-weight:bold;border-radius:4px}.container{display:flex;flex:1;flex-direction:column}@media(min-width: 768px){.container{flex-direction:row}}textarea{flex:1;background:#1e1e1e;color:#9cdcfe;border:none;padding:15px;font-size:16px;outline:none;resize:none}pre{flex:1;padding:15px;margin:0;overflow-y:auto;color:#4af626;background:#0d0d0d;border-top:1px solid #333}</style></head><body><div class="header"><div class="header-left"><img src="/icon.png" class="logo" onerror="this.style.display='none'"><h1>Aether Studio</h1></div><button onclick="run()">▶ SPUSTIT</button></div><div class="container"><textarea id="code" spellcheck="false">// Vitej v Mega Updatu Aetheru!
let hrac = { jmeno: "Aether", sila: 9000 }
print("Hrac:")
print(hrac["jmeno"])

let zbrane = ["Mec", "Luk", "Magie"]
for zbran in zbrane { print(zbran) }
</textarea><pre id="output">Čekám...</pre></div><script>async function run(){let o=document.getElementById('output');o.style.color='#888';o.innerText="Kompiluji...";let r=await fetch('/run',{method:'POST',body:document.getElementById('code').value});o.style.color='#4af626';o.innerText=await r.text();}</script></body></html>"#;
                    let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", html).as_bytes());
                } else if req.starts_with("POST /run ") {
                    if let Some(idx) = req.find("\r\n\r\n") {
                        let code = req[idx+4..].trim_matches(char::from(0));
                        let lexer = crate::lexer::Lexer::new(code); let mut parser = crate::parser::Parser::new(lexer); let program = parser.parse_program();
                        let mut env = crate::evaluator::Environment::new(false);
                        crate::evaluator::eval_program(&program, &mut env);
                        let mut out = env.output.join("\n"); if out.is_empty() { out = "OK".to_string(); }
                        let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}", out).as_bytes());
                    }
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--studio".to_string()) { serve_studio(); std::process::exit(0); }
    let config = Config::load();
    if args.contains(&"--edit-config".to_string()) { let _ = Command::new(&config.editor).arg(&format!("{}/.aether_config", env::var("HOME").unwrap_or_else(|_| ".".to_string()))).status(); std::process::exit(0); }
    
    let mut filename = "";
    let mut script_args = Vec::new();
    let mut past_file = false;
    for arg in args.iter().skip(1) { 
        if !arg.starts_with("--") && filename.is_empty() { filename = arg; past_file = true; continue; }
        if past_file { script_args.push(evaluator::Object::StringObj(arg.clone())); }
    }
    
    if filename.is_empty() { eprintln!("Chyba: Musíš zadat cestu k .ae souboru!"); std::process::exit(1); }
    if !filename.ends_with(".ae") { eprintln!("🛑 Formát musí být .ae"); std::process::exit(1); }

    let contents = fs::read_to_string(filename).unwrap_or_else(|_| { eprintln!("Nelze přečíst!"); std::process::exit(1); });
    let mut env = evaluator::Environment::new(args.contains(&"--stop-shut-up".to_string()) || config.auto_verbose);
    
    // VLOŽENÍ CLI ARGUMENTŮ DO AETHERU!
    env.set("SYS_ARGS".to_string(), evaluator::Object::Array(script_args));

    let lexer = lexer::Lexer::new(&contents);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();

    evaluator::eval_program(&program, &mut env);
}
