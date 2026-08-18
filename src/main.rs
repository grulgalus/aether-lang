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

fn get_config_path() -> String { format!("{}/.aether-config", env::var("HOME").unwrap_or_else(|_| ".".to_string())) }
fn get_lib_path() -> String { format!("{}/.aether-lib", env::var("HOME").unwrap_or_else(|_| ".".to_string())) }

fn ensure_dirs_exist() {
    let _ = fs::create_dir_all(get_lib_path());
    let path = get_config_path();
    if !std::path::Path::new(&path).exists() {
        let _ = fs::write(&path, "language-of-aether=en\nauto-open-file-if-is-broken=off\nauto-stop-shut-up-compilator=off\ndefault-editor-command=nano\n");
    }
}

fn tr(en: &str, cs: &str) -> String {
    if let Ok(c) = fs::read_to_string(get_config_path()) {
        if c.contains("language-of-aether=cs") { return cs.to_string(); }
    }
    en.to_string()
}

fn security_check(kód: &str) -> Result<(), String> {
    let nebezpecne = ["cmd(\"rm", "cmd('rm", "cmd(\"del", "cmd('del", "cmd(\"mkfs", "cmd(\"format"];
    for bad in nebezpecne {
        if kód.contains(bad) { return Err(format!("{} {}", tr("MALWARE DETECTED! Code contains forbidden command:", "DETEKOVÁN MALWARE! Kód obsahuje zakázaný příkaz:"), bad)); }
    }
    Ok(())
}

fn amp_install(balicek: &str) {
    println!("📦 {} '{}'...", tr("Fetching package", "Stahuji balíček"), balicek);
    let url = format!("https://raw.githubusercontent.com/aether-lang/amp-hub/main/{}.ae", balicek);
    if let Ok(out) = Command::new("curl").arg("-s").arg("-f").arg(&url).output() {
        if out.status.success() {
            let kod = String::from_utf8_lossy(&out.stdout).to_string();
            if let Err(vir) = security_check(&kod) { println!("❌ {}!", vir); return; }
            let cil = format!("{}/{}.ae", get_lib_path(), balicek);
            let _ = fs::write(&cil, kod);
            println!("✅ {} '{}' {}!", tr("Package", "Balíček"), balicek, tr("was successfully installed", "byl úspěšně nainstalován"));
        } else { println!("❌ {} '{}' {}!", tr("Package", "Balíček"), balicek, tr("not found", "nenalezen")); }
    } else { println!("❌ {}!", tr("Failed to connect to Internet", "Nepodařilo se připojit k internetu")); }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("🔮 Aether Lang v1.0.0-beta");
        println!("{}", tr("Type 'aether --help' for manual and syntax guide.", "Napiš 'aether --help' pro manuál a průvodce syntaxí."));
        return;
    }

    ensure_dirs_exist();
    let prikaz = args[1].as_str();

    match prikaz {
        "--help" => {
            println!("{}\n", tr("🔮 AETHER LANG - MANUAL", "🔮 AETHER LANG - MANUÁL"));
            println!("{}", tr("--- CLI COMMANDS ---", "--- PŘÍKAZY TERMINÁLU ---"));
            println!("  aether <file.ae>      - {}", tr("Run script", "Spustí skript"));
            println!("  aether check <file>   - {}", tr("Scan for malware & syntax errors", "Zkontroluje kód na viry a syntaxi (BEZ spuštění)"));
            println!("  aether install <pkg>  - {}", tr("Install AMP package (Aether Module Package)", "Nainstaluje AMP balíček (Aether Module Package)"));
            println!("  aether fmt <file>     - {}", tr("Format code", "Automaticky naformátuje kód"));
            println!("  aether --studio       - {}", tr("Launch Web IDE", "Spustí webové IDE"));
            println!("  aether --config       - {}", tr("Show config", "Zobrazí konfiguraci"));
            println!("  aether --edit-config  - {}", tr("Edit config", "Upraví konfiguraci"));
            println!("  aether --help         - {}", tr("Show this manual", "Zobrazí tento manuál"));
            
            println!("\n{}", tr("--- SYNTAX & EXAMPLES ---", "--- SYNTAXE A PŘÍKLADY ---"));
            println!("{}", tr("Variables:  let name = \"Aether\"; let x = 10", "Proměnné:  let jmeno = \"Aether\"; let x = 10"));
            println!("{}", tr("Printing:   print(\"Hello \" + name)", "Výpis:     print(\"Ahoj \" + jmeno)"));
            println!("{}", tr("Arrays:     let arr = [1, 2, 3]; push(arr, 4)", "Pole:      let pole = [1, 2, 3]; push(pole, 4)"));
            println!("{}", tr("Dicts:      let obj = { x: 10, y: 20 }", "Slovníky:  let hrac = { sila: 9000, zbran: \"Mec\" }"));
            println!("{}", tr("If/Else:    if x > 5 { print(\"Big\") } else { print(\"Small\") }", "Podmínky:  if x > 5 { print(\"Velke\") } else { print(\"Male\") }"));
            println!("{}", tr("Loops:      for item in arr { print(item) }", "Cykly:     for polozka in pole { print(polozka) }"));
            println!("{}", tr("            while x < 20 { x = x + 1 }", "           while x < 20 { x = x + 1 }"));
            
            println!("\n{}", tr("--- BUILT-IN FUNCTIONS ---", "--- VESTAVĚNÉ FUNKCE ---"));
            println!("  read(file), write(file, text) - {}", tr("File I/O", "Čtení a zápis souborů"));
            println!("  serve(port, html)             - {}", tr("Start Web Server", "Spustí Webový Server na daném portu"));
            println!("  cmd(command)                  - {}", tr("Run OS terminal command", "Spustí příkaz v terminálu (Termux/Linux)"));
            println!("  fetch(url)                    - {}", tr("GET request to internet", "Stáhne textová data z internetu"));
            println!("  discord(webhook, msg)         - {}", tr("Send Discord message", "Pošle zprávu do Discord kanálu přes Webhook"));
            println!("  exec(code_string)             - {}", tr("Run dynamic Aether code", "Zkompiluje a spustí Aether kód z textu za běhu"));
            println!("  rand(), time(), sleep(ms), clear(), len(), upper(), split(), replace()");
            return;
        },
        "install" => { if args.len() > 2 { amp_install(&args[2]); } else { println!("❌ {}", tr("Specify package name!", "Zadej název balíčku!")); } return; },
        "fmt" => { println!("✨ {} (Aether v1.1)!", tr("Formatter is in development", "FMT Formatter se vyvíjí a dorazí v dalším updatu")); return; },
        "check" => {
            if args.len() > 2 {
                let soubor = &args[2];
                println!("🔍 {} {}...", tr("Scanning file", "Skener analyzuje soubor"), soubor);
                if let Ok(obsah) = fs::read_to_string(soubor) {
                    let start = Instant::now();
                    if let Err(vir) = security_check(&obsah) { println!("❌ {}\n🛑 {}", vir, tr("File is DANGEROUS!", "Soubor je NEBEZPEČNÝ!")); return; }
                    let lexer = Lexer::new(&obsah); let mut parser = Parser::new(lexer); let _ = parser.parse_program(); 
                    println!("✅ {} ({} ms)", tr("File is SAFE and syntactically correct", "Kód je čistý, bezpečný a připraven ke spuštění"), start.elapsed().as_millis());
                } else { println!("❌ {}: '{}'", tr("FILE NOT FOUND", "SOUBOR NENALEZEN"), soubor); }
            } else { println!("❌ {}", tr("Specify file to check!", "Zadej soubor ke kontrole!")); }
            return;
        },
        "--config" => { println!("🔧 {}:", tr("Config", "Konfigurace")); println!("{}", fs::read_to_string(get_config_path()).unwrap_or_default()); return; },
        "--edit-config" => { let _ = Command::new("nano").arg(get_config_path()).status(); return; },
        "--studio" => { println!("🚀 {} http://127.0.0.1:8765", tr("Launching Aether Studio on", "Spouštím Aether Studio na")); start_studio(); return; },
        _ => {} 
    }

    let stop_shut_up = args.contains(&"--stop-shut-up".to_string());
    
    let start = Instant::now();
    let obsah = match fs::read_to_string(prikaz) { Ok(c) => c, Err(_) => { if !stop_shut_up { println!("🛑 {}: '{}'", tr("FILE NOT FOUND", "SOUBOR NENALEZEN"), prikaz); } return; } };
    if let Err(vir) = security_check(&obsah) { println!("🛡️ {}", vir); return; }
    
    let lexer = Lexer::new(&obsah); let mut parser = Parser::new(lexer); let program = parser.parse_program();
    let mut env = Environment::new(); let vysledek = eval_program(&program, &mut env);
    
    if let Object::Error(e) = vysledek {
        if !stop_shut_up { println!("{}", e); }
        if let Ok(c) = fs::read_to_string(get_config_path()) { if c.contains("auto-open-file-if-is-broken=on") { let _ = Command::new("nano").arg(prikaz).status(); } }
    }
    if !stop_shut_up { println!("\n⏱️ {}: {:.2?}", tr("Finished in", "Dokončeno za"), start.elapsed()); }
}

fn start_studio() {
    let listener = TcpListener::bind("127.0.0.1:8765").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 4096]; if stream.read(&mut buffer).is_err() { continue; }
            let req = String::from_utf8_lossy(&buffer);
            if req.starts_with("POST /run") {
                if let Some(body_start) = req.find("\r\n\r\n") {
                    let code = req[body_start + 4..].trim_matches(char::from(0)).trim().to_string();
                    if let Err(vir) = security_check(&code) {
                        let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}", vir).as_bytes()); continue;
                    }
                    let lexer = Lexer::new(&code); let mut parser = Parser::new(lexer); let prog = parser.parse_program(); let mut env = Environment::new();
                    eval_program(&prog, &mut env);
                    let mut vys = env.output.join("\n"); if vys.is_empty() { vys = "OK".to_string(); }
                    let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}", vys).as_bytes());
                }
            } else { let html = include_str!("../aether-studio.html").to_string(); let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}", html).as_bytes()); }
        }
    }
}
