use std::env;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use std::process::Command;

mod lexer;
mod ast;
mod parser;
mod evaluator;

// 0. CHYTRÁ DETEKCE EDITORU
fn detect_editor() -> String {
    // 1. Zkusíme zjistit, jestli má uživatel nastavenou systémovou proměnnou EDITOR
    if let Ok(ed) = env::var("EDITOR") {
        if !ed.is_empty() { return ed; }
    }
    // 2. Pokud ne, vyzkoušíme najít nejpoužívanější editory pomocí příkazu 'which'
    let editors = ["nano", "vim", "nvim", "vi", "emacs"];
    for ed in editors.iter() {
        if let Ok(out) = Command::new("which").arg(ed).output() {
            if out.status.success() { return ed.to_string(); }
        }
    }
    // 3. Fallback, kdyby selhalo úplně všechno
    "nano".to_string() 
}

// 1. KONFIGURAČNÍ STRUKTURA
struct Config {
    language: String,
    auto_open_broken: bool,
    auto_verbose: bool,
    editor: String,
}

impl Config {
    fn load() -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_path = format!("{}/.aether_config", home);
        
        let mut conf = Config {
            language: "en".to_string(),
            auto_open_broken: false,
            auto_verbose: false,
            editor: detect_editor(), // Automatická detekce!
        };

        if let Ok(content) = fs::read_to_string(&config_path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val = parts[1].trim();
                    match key {
                        "language-of-aether" => conf.language = val.to_string(),
                        "auto-open-file-if-is-broken" => conf.auto_open_broken = val == "on",
                        "auto-stop-shut-up-compilator" => conf.auto_verbose = val == "on",
                        "default-editor-command" => conf.editor = val.to_string(),
                        _ => {}
                    }
                }
            }
        } else {
            // Vytvoření výchozího souboru s dynamicky nalezeným editorem a novými výchozími hodnotami
            let default_cfg = format!(
                "language-of-aether=en\nauto-open-file-if-is-broken=off\nauto-stop-shut-up-compilator=off\ndefault-editor-command={}\n",
                conf.editor
            );
            let _ = fs::write(&config_path, default_cfg);
        }
        conf
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::load();

    if args.contains(&"--edit-config".to_string()) {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_path = format!("{}/.aether_config", home);
        println!("⚙️ Otevírám konfiguraci přes editor: {}", config.editor);
        let _ = Command::new(&config.editor).arg(&config_path).status();
        std::process::exit(0);
    }

    if args.len() < 2 { eprintln!("Použití: aether <soubor.ae> [--stop-shut-up] [--be-insane] [--edit-config]"); std::process::exit(1); }
    
    let ukecany_rezim = args.contains(&"--stop-shut-up".to_string()) || config.auto_verbose;
    let insane_mode = args.contains(&"--be-insane".to_string());
    
    let mut filename = "";
    for arg in args.iter().skip(1) { if !arg.starts_with("--") { filename = arg; break; } }
    if filename.is_empty() { eprintln!("Chyba: Musíš zadat cestu k .ae souboru!"); std::process::exit(1); }

    if !filename.ends_with(".ae") {
        let spatna_pripona = filename.split('.').last().unwrap_or("bez_pripony");
        eprintln!("🛑 [KRITICKÁ CHYBA FORMÁTU] Co to na mě zkoušíš za formát? '.{}'?!", spatna_pripona);
        eprintln!("Aether přijímá výhradně a pouze čistokrevné '.ae' skripty. Nejsme v cirkuse!");
        std::process::exit(1);
    }

    let contents = match fs::read_to_string(filename) { Ok(c) => c, Err(_) => { eprintln!("Chyba: Nelze přečíst soubor '{}'", filename); std::process::exit(1); } };
    let exec_start = Instant::now();
            
    let lexer = lexer::Lexer::new(&contents);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();

    if program.statements.is_empty() && !contents.trim().is_empty() {
        eprintln!("🛑 [SYNTAX ERROR] Kompilátor nedokázal přečíst kód! Zřejmě jsi udělal hrubou chybu v syntaxi.");
        if config.auto_open_broken {
            eprintln!("🛠️ 'auto-open-file-if-is-broken' je ZAPNUTO. Otevírám {} pomocí {}...", filename, config.editor);
            let _ = Command::new(&config.editor).arg(filename).status();
        } else {
            eprintln!("💡 Tip: Zapni si 'auto-open-file-if-is-broken=on' v '--edit-config', aether ti ho sám otevře k opravě!");
        }
        std::process::exit(1);
    }
        
    if insane_mode {
        println!("\nerror[E0596]: cannot borrow `reality` as mutable, as it is not declared as mutable");
        thread::sleep(Duration::from_millis(800));
        println!("\nTraceback (most recent call last):\n  File \"aether_core.py\", line 666\nKeyboardInterrupt: User pressed CTRL+C but the AI refuses to die!");
        thread::sleep(Duration::from_millis(800));
        println!("\n[FATAL] Segmentation fault (core dumped) in Android Bionic libc.");
        println!("Warning: The compiler is experiencing existential dread...\n\n...just kidding. Všechno je v pohodě.\n");
    } else if ukecany_rezim {
        let line_count = contents.lines().count();
        let file_size = contents.len();
        let os_info = env::consts::OS;
        let arch_info = env::consts::ARCH;
        
        println!("==================================================");
        println!("🌌 AETHER COMPILER DIAGNOSTICS & SYSTEM INFO");
        println!("==================================================");
        println!("📌 Verze kompilátoru: v0.1.0-masterclass");
        println!("🌍 Jazyk Aetheru:      {}", config.language);
        println!("⚙️  Výchozí editor:    {}", config.editor);
        println!("🖥️  Cílový systém:     {} ({})", os_info, arch_info);
        println!("📂 Zpracováván soubor: {}", filename);
        println!("📊 Statistiky kódu:   {} řádků | {} bytů", line_count, file_size);
        println!("🏷️  Typ souboru:       .ae (Ověřeno & Validní)");
        println!("==================================================");
        println!("✨ Fáze 1: Lexikální a syntaktická analýza...");
        println!("🚀 Fáze 2: Spouštím virtuální stroj Aetheru...\n");
    }
    
    let mut env = evaluator::Environment::new(ukecany_rezim && !insane_mode);
    let result = evaluator::eval_program(&program, &mut env);
    
    if insane_mode { println!("\n========================================\n[CRITICAL ERROR] Task failed successfully.\nV RAM zůstalo viset 128 GB dat a tvůj procesor hoří. Hodně štěstí."); std::process::exit(0); }

    if ukecany_rezim && !insane_mode {
        let exec_duration = exec_start.elapsed();
        println!("\n==================================================");
        println!("--- VÝSLEDEK BĚHU PROGRAMU ---");
        match result {
            evaluator::Object::Number(n) => println!("Vrácená hodnota: {}", n),
            evaluator::Object::StringObj(s) => println!("Vrácená hodnota: {}", s),
            evaluator::Object::Boolean(b) => println!("Vrácená hodnota: {}", b),
            evaluator::Object::Array(arr) => println!("Vrácená hodnota: Pole (obsahuje {} polozek)", arr.len()),
            evaluator::Object::Null => println!("Program proběhl, ale nevrátil nic (Null)."),
        }
        println!("⏱️  Celkový čas běhu: {:?}", exec_duration);
        println!("✅ Systém úspěšně ukončen.");
        println!("==================================================");
    }
}
