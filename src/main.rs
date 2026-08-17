use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Instant;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::evaluator::{eval_program, Environment, Object};

mod lexer;
mod parser;
mod ast;
mod evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("🔮 Aether Lang v1.0.0");
        println!("Použití:");
        println!("  aether <soubor.ae>    - Spustí skript");
        println!("  aether --studio       - Spustí webové IDE");
        println!("  aether --config       - Zobrazí konfiguraci");
        println!("  aether --help         - Zobrazí nápovědu");
        return;
    }

    let prikaz = args[1].as_str();

    match prikaz {
        "--help" => {
            println!("Nápověda pro Aether:");
            println!("Napiš 'let x = 10' pro proměnnou.");
            println!("Napiš 'print(x)' pro výpis.");
            return;
        },
        "--config" => {
            println!("🔧 Konfigurace Aetheru: \n - Verze: 1.0.0 \n - Optimalizace: ZAPNUTO \n - JNI: ZAPNUTO");
            return;
        },
        "--edit-config" => {
            println!("(Tato funkce bude dostupná v dalším updatu!)");
            return;
        },
        "--studio" => {
            println!("🚀 Spouštím Aether Studio na http://127.0.0.1:8765");
            start_studio();
            return;
        },
        _ => {} // Pokud to není vlaječka, je to soubor
    }

    let start = Instant::now();
    let obsah = match fs::read_to_string(prikaz) {
        Ok(c) => c,
        Err(_) => {
            println!("🛑 KRITICKÁ CHYBA: Soubor '{}' neexistuje!", prikaz);
            return;
        }
    };
    
    let lexer = Lexer::new(&obsah);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    let mut env = Environment::new();
    let vysledek = eval_program(&program, &mut env);
    
    if let Object::Error(e) = vysledek {
        println!("{}", e);
    }
    
    println!("\n⏱️ Dokončeno za: {:.2?}", start.elapsed());
}

// 🌐 WEBOVÝ SERVER PRO AETHER STUDIO
fn start_studio() {
    let listener = match TcpListener::bind("127.0.0.1:8765") {
        Ok(l) => l,
        Err(_) => { println!("Nelze spustit server! Port 8765 je asi obsazený."); return; }
    };
    
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 4096];
            if stream.read(&mut buffer).is_err() { continue; }
            let request = String::from_utf8_lossy(&buffer);
            
            if request.starts_with("POST /run") {
                if let Some(body_start) = request.find("\r\n\r\n") {
                    let code = request[body_start + 4..].trim_matches(char::from(0)).trim().to_string();
                    
                    let lexer = Lexer::new(&code);
                    let mut parser = Parser::new(lexer);
                    let program = parser.parse_program();
                    let mut env = Environment::new();
                    
                    eval_program(&program, &mut env);
                    let mut vystup = env.output.join("\n");
                    if vystup.is_empty() { vystup = "OK (Bez výstupu)".to_string(); }
                    
                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}", vystup);
                    let _ = stream.write_all(response.as_bytes());
                }
            } else {
                let html = include_str!("../aether-studio.html").to_string(); // Předpokládá existenci HTML
                let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}", html);
                let _ = stream.write_all(response.as_bytes());
            }
        }
    }
}
