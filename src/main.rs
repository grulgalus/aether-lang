use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Instant;
use std::process::Command;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::evaluator::{eval_program, Environment, Object};

mod lexer;
mod parser;
mod ast;
mod evaluator;

// Cesta ke konfiguraci (v domovské složce Termuxu/Linuxu)
fn get_config_path() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.aether-config", home)
}

// Ujistíme se, že config existuje
fn ensure_config_exists() -> String {
    let path = get_config_path();
    if !std::path::Path::new(&path).exists() {
        let default_cfg = "language-of-aether=en\nauto-open-file-if-is-broken=off\nauto-stop-shut-up-compilator=off\ndefault-editor-command=nano\n";
        let _ = fs::write(&path, default_cfg);
    }
    path
}

// Přečtení konkrétní hodnoty z configu
fn read_config_value(key: &str) -> String {
    let path = ensure_config_exists();
    if let Ok(content) = fs::read_to_string(&path) {
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k.trim() == key { return v.trim().to_string(); }
            }
        }
    }
    if key == "default-editor-command" { return "nano".to_string(); }
    "off".to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("🔮 Aether Lang v1.0.0");
        println!("Použití:");
        println!("  aether <soubor.ae>    - Spustí skript");
        println!("  aether --studio       - Spustí webové IDE");
        println!("  aether --config       - Zobrazí konfiguraci");
        println!("  aether --edit-config  - Upraví konfiguraci");
        println!("  aether --help         - Zobrazí nápovědu");
        return;
    }

    let prikaz = args[1].as_str();
    ensure_config_exists();

    match prikaz {
        "--help" => { println!("Nápověda pro Aether..."); return; },
        "--config" => {
            let path = get_config_path();
            println!("🔧 Konfigurace Aetheru ({}):", path);
            println!("{}", fs::read_to_string(&path).unwrap_or_default());
            return;
        },
        "--edit-config" => {
            let path = get_config_path();
            let editor = read_config_value("default-editor-command");
            println!("📝 Otevírám konfiguraci v editoru: {}", editor);
            let _ = Command::new(&editor).arg(&path).status();
            return;
        },
        "--studio" => {
            println!("🚀 Spouštím Aether Studio na http://127.0.0.1:8765");
            start_studio();
            return;
        },
        _ => {} 
    }

    // Čteme config + vlaječky pro umlčení stopek atd.
    let stop_shut_up = args.contains(&"--stop-shut-up".to_string()) || read_config_value("auto-stop-shut-up-compilator") == "on";

    let start = Instant::now();
    let obsah = match fs::read_to_string(prikaz) {
        Ok(c) => c,
        Err(_) => {
            if !stop_shut_up { println!("🛑 KRITICKÁ CHYBA: Soubor '{}' neexistuje!", prikaz); }
            return;
        }
    };
    
    let lexer = Lexer::new(&obsah);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    let mut env = Environment::new();
    let vysledek = eval_program(&program, &mut env);
    
    if let Object::Error(e) = vysledek {
        if !stop_shut_up { println!("{}", e); }
        
        // ZLATÝ HŘEB: Pokud je kód rozbitý a máš nastavený auto-open, tak ti rovnou otevře Nano!
        if read_config_value("auto-open-file-if-is-broken") == "on" {
            let editor = read_config_value("default-editor-command");
            println!("🔧 Otevírám rozbitý soubor v editoru {}...", editor);
            let _ = Command::new(&editor).arg(prikaz).status();
        }
    }
    
    if !stop_shut_up {
        println!("\n⏱️ Dokončeno za: {:.2?}", start.elapsed());
    }
}

// 🌐 WEBOVÝ SERVER PRO AETHER STUDIO
fn start_studio() {
    let listener = match TcpListener::bind("127.0.0.1:8765") { Ok(l) => l, Err(_) => { println!("Port 8765 je obsazený."); return; } };
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 4096];
            if stream.read(&mut buffer).is_err() { continue; }
            let request = String::from_utf8_lossy(&buffer);
            if request.starts_with("POST /run") {
                if let Some(body_start) = request.find("\r\n\r\n") {
                    let code = request[body_start + 4..].trim_matches(char::from(0)).trim().to_string();
                    let lexer = Lexer::new(&code); let mut parser = Parser::new(lexer); let program = parser.parse_program(); let mut env = Environment::new();
                    eval_program(&program, &mut env);
                    let mut vystup = env.output.join("\n"); if vystup.is_empty() { vystup = "OK (Bez výstupu)".to_string(); }
                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}", vystup);
                    let _ = stream.write_all(response.as_bytes());
                }
            } else {
                let html = include_str!("../aether-studio.html").to_string();
                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}", html);
                let _ = stream.write_all(response.as_bytes());
            }
        }
    }
}
