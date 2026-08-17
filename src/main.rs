use std::env;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};

mod lexer;
mod ast;
mod parser;
mod evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 { eprintln!("Použití: aether <soubor.ae> [--stop-shut-up] [--be-insane]"); std::process::exit(1); }
    
    let ukecany_rezim = args.contains(&"--stop-shut-up".to_string());
    let insane_mode = args.contains(&"--be-insane".to_string());
    
    let mut filename = "";
    for arg in args.iter().skip(1) { if arg != "--stop-shut-up" && arg != "--be-insane" { filename = arg; break; } }
    if filename.is_empty() { eprintln!("Chyba: Musíš zadat cestu k .ae souboru!"); std::process::exit(1); }

    let contents = match fs::read_to_string(filename) { Ok(c) => c, Err(_) => { eprintln!("Chyba: Nelze přečíst soubor '{}'", filename); std::process::exit(1); } };
    
    // Zapínáme stopky přímo v jádru!
    let exec_start = Instant::now();
        
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
        let ext = filename.split('.').last().unwrap_or("neznámý");
        
        // Získání systémových informací přes Rust!
        let os_info = env::consts::OS;
        let arch_info = env::consts::ARCH;
        
        println!("==================================================");
        println!("🌌 AETHER COMPILER DIAGNOSTICS & SYSTEM INFO");
        println!("==================================================");
        println!("📌 Verze kompilátoru: v0.1.0-masterclass");
        println!("🖥️  Cílový systém:     {} ({})", os_info, arch_info);
        println!("📂 Zpracováván soubor: {}", filename);
        println!("📊 Statistiky kódu:   {} řádků | {} bytů", line_count, file_size);
        println!("🏷️  Typ souboru:       .{}", ext);
        println!("==================================================");
        println!("✨ Fáze 1: Lexikální a syntaktická analýza...");
        println!("🚀 Fáze 2: Spouštím virtuální stroj Aetheru...\n");
    }
            
    let lexer = lexer::Lexer::new(&contents);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();
    
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
        // Vytiskneme celkový čas běhu Aetheru!
        println!("⏱️  Celkový čas běhu: {:?}", exec_duration);
        println!("✅ Systém úspěšně ukončen.");
        println!("==================================================");
    }
}
