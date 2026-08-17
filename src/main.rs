use std::env;
use std::fs;

mod lexer;
mod ast;
mod parser;
mod evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Pokud nezadáme ani soubor, vypíšeme nápovědu
    if args.len() < 2 {
        eprintln!("Použití: aether <soubor.ae> [--stop-shut-up]");
        std::process::exit(1);
    }
    
    // Detekujeme tvůj příkaz pro ukecanou verzi (Verbose mode)
    let ukecany_rezim = args.contains(&"--stop-shut-up".to_string());
    
    // Najdeme jméno souboru (ignorujeme argument --stop-shut-up)
    let mut filename = "";
    for arg in args.iter().skip(1) {
        if arg != "--stop-shut-up" {
            filename = arg;
            break;
        }
    }

    if filename.is_empty() {
        eprintln!("Chyba: Musíš zadat cestu k .ae souboru!");
        std::process::exit(1);
    }

    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Chyba: Nelze přečíst soubor '{}'", filename);
            std::process::exit(1);
        }
    };
        
    // Vypisuje jen když jsme to povolili!
    if ukecany_rezim {
        println!("🌌 Aether Compiler v0.1.0");
        println!("📂 Načítám soubor: {}", filename);
        println!("✨ Kód načten. Lexikální a syntaktická analýza...");
    }
            
    let lexer = lexer::Lexer::new(&contents);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();
    
    if ukecany_rezim {
        println!("\n🚀 Spouštím virtuální stroj Aetheru...\n");
    }
    
    let mut env = evaluator::Environment::new();
    let result = evaluator::eval_program(&program, &mut env);
    
    // Závěrečný výpis se ukáže taky jen v ukecaném režimu
    if ukecany_rezim {
        println!("--- VÝSLEDEK BĚHU PROGRAMU ---");
        match result {
            evaluator::Object::Number(n) => println!("Vrácená hodnota: {}", n),
            evaluator::Object::StringObj(s) => println!("Vrácená hodnota: {}", s),
            evaluator::Object::Boolean(b) => println!("Vrácená hodnota: {}", b),
            evaluator::Object::Null => println!("Program proběhl, ale nevrátil nic (Null)."),
        }
        println!("\n✅ Hotovo.");
    }
}
