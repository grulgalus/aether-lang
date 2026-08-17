use std::env;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use std::process::Command;
use std::io::{Read, Write};
use std::net::TcpListener;

mod lexer;
mod ast;
mod parser;
mod evaluator;

fn detect_editor() -> String {
    if let Ok(ed) = env::var("EDITOR") { if !ed.is_empty() { return ed; } }
    for ed in ["nano", "vim", "nvim", "vi", "emacs"].iter() { if let Ok(out) = Command::new("which").arg(ed).output() { if out.status.success() { return ed.to_string(); } } }
    "nano".to_string() 
}

struct Config { language: String, auto_open_broken: bool, auto_verbose: bool, editor: String }
impl Config {
    fn load() -> Self {
        let config_path = format!("{}/.aether_config", env::var("HOME").unwrap_or_else(|_| ".".to_string()));
        let mut conf = Config { language: "en".to_string(), auto_open_broken: false, auto_verbose: false, editor: detect_editor() };
        if let Ok(content) = fs::read_to_string(&config_path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 { match parts[0].trim() { "language-of-aether" => conf.language = parts[1].trim().to_string(), "auto-open-file-if-is-broken" => conf.auto_open_broken = parts[1].trim() == "on", "auto-stop-shut-up-compilator" => conf.auto_verbose = parts[1].trim() == "on", "default-editor-command" => conf.editor = parts[1].trim().to_string(), _ => {} } }
            }
        } else { let _ = fs::write(&config_path, format!("language-of-aether=en\nauto-open-file-if-is-broken=off\nauto-stop-shut-up-compilator=off\ndefault-editor-command={}\n", conf.editor)); }
        conf
    }
}

// ==========================================
// ZABUDOVANÝ WEBOVÝ SERVER: AETHER STUDIO!
// ==========================================
fn serve_studio() {
    let listener = TcpListener::bind("127.0.0.1:8765").expect("Port 8765 je zabrany!");
    println!("==================================================");
    println!("🎨 AETHER STUDIO BĚŽÍ!");
    println!("👉 http://127.0.0.1:8765 👈");
    println!("(Kompilátor zastavíš zkratkou CTRL+C)");
    println!("==================================================");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 32768];
            if let Ok(bytes_read) = stream.read(&mut buffer) {
                let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                
                // 1. Zpracování požadavku na IKONU!
                if request.starts_with("GET /icon.png ") {
                    // Zkusíme načíst tvoji ikonu ze složky
                    if let Ok(img) = fs::read("aether_space_icon.png") {
                        let header = format!("HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\n\r\n", img.len());
                        let _ = stream.write_all(header.as_bytes());
                        let _ = stream.write_all(&img);
                    } else {
                        // Pokud ikonu ještě nemáš staženou, nic to nerozbije
                        let _ = stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n");
                    }
                }
                // 2. Servírujeme HTML okno Aether Studia (přidáno logo!)
                else if request.starts_with("GET / ") || request.starts_with("GET / HTTP") {
                    let html = r#"<!DOCTYPE html><html><head><meta charset="utf-8">
                        <title>Aether Studio</title>
                        <link rel="icon" type="image/png" href="/icon.png">
                        <meta name="viewport" content="width=device-width, initial-scale=1.0"><style>
                        body { margin: 0; background: #121212; color: #d4d4d4; font-family: monospace; display: flex; flex-direction: column; height: 100vh; }
                        .header { background: #1e1e1e; padding: 15px; display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid #333; }
                        .header-left { display: flex; align-items: center; gap: 15px; }
                        .logo { width: 40px; height: 40px; border-radius: 8px; box-shadow: 0 0 10px rgba(0, 255, 255, 0.2); }
                        .header h1 { margin: 0; font-size: 22px; color: #4af626; text-shadow: 0 0 5px rgba(74, 246, 38, 0.4); }
                        button { background: #00bcd4; color: #121212; border: none; padding: 10px 20px; font-size: 16px; cursor: pointer; font-weight: bold; border-radius: 4px; transition: 0.2s;}
                        button:hover { background: #0097a7; transform: scale(1.05); }
                        .container { display: flex; flex: 1; flex-direction: column; }
                        @media(min-width: 768px) { .container { flex-direction: row; } }
                        textarea { flex: 1; background: #1e1e1e; color: #9cdcfe; border: none; padding: 15px; font-family: monospace; font-size: 16px; outline: none; resize: none; border-right: 1px solid #333; }
                        pre { flex: 1; padding: 15px; margin: 0; overflow-y: auto; color: #4af626; background: #0d0d0d; font-size: 15px; border-top: 1px solid #333; }
                    </style></head><body>
                        <div class="header">
                            <div class="header-left">
                                <img src="/icon.png" alt="Aether Logo" class="logo" onerror="this.style.display='none'">
                                <h1>Aether Studio</h1>
                            </div>
                            <button onclick="run()">▶ SPUSTIT KÓD</button>
                        </div>
                        <div class="container">
                            <textarea id="code" spellcheck="false">// Vitej v Aether Studiu!
// Kompilator verze 0.1.0

print("Ahoj svete!")
let stesti = rand()

if stesti > 50 {
    print("Dnesni den bude plny kodu!")
} else {
    print("Dneska radsi bez ven!")
}
</textarea>
                            <pre id="output">Čekám na spuštění programu...</pre>
                        </div>
                        <script>
                            async function run() {
                                let out = document.getElementById('output');
                                out.style.color = '#888'; out.innerText = "Kompiluji...";
                                let code = document.getElementById('code').value;
                                let res = await fetch('/run', { method: 'POST', body: code });
                                out.style.color = '#4af626'; out.innerText = await res.text();
                            }
                        </script>
                    </body></html>"#;
                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}", html);
                    let _ = stream.write_all(response.as_bytes());
                } 
                else if request.starts_with("POST /run ") {
                    if let Some(idx) = request.find("\r\n\r\n") {
                        let code = request[idx+4..].trim_matches(char::from(0));
                        let lexer = crate::lexer::Lexer::new(code);
                        let mut parser = crate::parser::Parser::new(lexer);
                        let program = parser.parse_program();
                        let mut env = crate::evaluator::Environment::new(false);
                        
                        crate::evaluator::eval_program(&program, &mut env);
                        
                        let mut final_output = env.output.join("\n");
                        if final_output.is_empty() { final_output = "(Program běžel správně, ale nic nevypsal)".to_string(); }
                        
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}", final_output);
                        let _ = stream.write_all(response.as_bytes());
                    }
                }
            }
        }
    }
}
// ==========================================

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--studio".to_string()) { serve_studio(); std::process::exit(0); }

    let config = Config::load();
    if args.contains(&"--edit-config".to_string()) { let _ = Command::new(&config.editor).arg(&format!("{}/.aether_config", env::var("HOME").unwrap_or_else(|_| ".".to_string()))).status(); std::process::exit(0); }
    if args.len() < 2 { eprintln!("Použití: aether <soubor.ae> [--stop-shut-up] [--be-insane] [--edit-config] [--studio]"); std::process::exit(1); }
    let ukecany_rezim = args.contains(&"--stop-shut-up".to_string()) || config.auto_verbose;
    let insane_mode = args.contains(&"--be-insane".to_string());
    
    let mut filename = "";
    for arg in args.iter().skip(1) { if !arg.starts_with("--") { filename = arg; break; } }
    if filename.is_empty() { eprintln!("Chyba: Musíš zadat cestu k .ae souboru!"); std::process::exit(1); }
    if !filename.ends_with(".ae") { eprintln!("🛑 [KRITICKÁ CHYBA FORMÁTU] Nesprávná přípona. Vyžadováno '.ae'"); std::process::exit(1); }

    let contents = match fs::read_to_string(filename) { Ok(c) => c, Err(_) => { eprintln!("Chyba: Nelze přečíst soubor '{}'", filename); std::process::exit(1); } };
    let exec_start = Instant::now();
            
    let lexer = lexer::Lexer::new(&contents);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();

    if program.statements.is_empty() && !contents.trim().is_empty() {
        eprintln!("🛑 [SYNTAX ERROR] Kompilátor nedokázal přečíst kód!");
        if config.auto_open_broken { let _ = Command::new(&config.editor).arg(filename).status(); }
        std::process::exit(1);
    }
        
    if insane_mode { println!("\nerror[E0596]: cannot borrow `reality` as mutable... just kidding.\n"); } 
    else if ukecany_rezim { println!("🚀 Spouštím virtuální stroj...\n"); }
    
    let mut env = evaluator::Environment::new(ukecany_rezim && !insane_mode);
    let result = evaluator::eval_program(&program, &mut env);

    if ukecany_rezim && !insane_mode { println!("⏱️ Celkový čas běhu: {:?}", exec_start.elapsed()); }
}
