use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

mod lexer;
mod ast;
mod parser;
mod evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Použití: aether <soubor.ae> [--stop-shut-up] [--be-insane]");
        std::process::exit(1);
    }
    
    let ukecany_rezim = args.contains(&"--stop-shut-up".to_string());
    let insane_mode = args.contains(&"--be-insane".to_string());
    
    let mut filename = "";
    for arg in args.iter().skip(1) {
        if arg != "--stop-shut-up" && arg != "--be-insane" {
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
        
    // --- EASTER EGG: INSANE MODE ---
    if insane_mode {
        println!("\nerror[E0596]: cannot borrow `reality` as mutable, as it is not declared as mutable");
        println!("  --> src/universe.rs:42:0");
        println!("   |");
        println!("42 |     let reality = universe.exist();");
        println!("   |         ------- help: consider changing this to be mutable: `mut reality`");
        println!("   = note: the borrow checker is currently crying in the corner.");
        
        thread::sleep(Duration::from_millis(800));
        
        println!("\nTraceback (most recent call last):");
        println!("  File \"aether_core.py\", line 666, in <module>");
        println!("    import skynet");
        println!("KeyboardInterrupt: User pressed CTRL+C but the AI refuses to die!");
        
        thread::sleep(Duration::from_millis(800));

        println!("\n[FATAL] Segmentation fault (core dumped) in Android Bionic libc.");
        println!("java.lang.NullPointerException: Object reference not set to an instance of an object.");
        println!("Warning: The compiler is experiencing existential dread...");
        
        thread::sleep(Duration::from_millis(1500));
        println!("\n...just kidding. Všechno je v pohodě. Spouštím tvůj kód:\n");
    } else if ukecany_rezim {
        println!("🌌 Aether Compiler v0.1.0");
        println!("📂 Načítám soubor: {}", filename);
        println!("✨ Kód načten. Lexikální a syntaktická analýza...");
    }
            
    let lexer = lexer::Lexer::new(&contents);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();
    
    if ukecany_rezim && !insane_mode {
        println!("\n🚀 Spouštím virtuální stroj Aetheru...\n");
    }
    
    let mut env = evaluator::Environment::new();
    let result = evaluator::eval_program(&program, &mut env);
    
    if insane_mode {
        println!("\n========================================");
        println!("[CRITICAL ERROR] Task failed successfully.");
        println!("V RAM zůstalo viset 128 GB dat a tvůj procesor hoří. Hodně štěstí.");
        std::process::exit(0);
    }

    if ukecany_rezim && !insane_mode {
        println!("--- VÝSLEDEK BĚHU PROGRAMU ---");
        match result {
            evaluator::Object::Number(n) => println!("Vrácená hodnota: {}", n),
            evaluator::Object::StringObj(s) => println!("Vrácená hodnota: {}", s),
            evaluator::Object::Boolean(b) => println!("Vrácená hodnota: {}", b),
            // TADY JE TA OPRAVA! Přidána podpora pro návrat Pole.
            evaluator::Object::Array(arr) => println!("Vrácená hodnota: Pole (obsahuje {} polozek)", arr.len()),
            evaluator::Object::Null => println!("Program proběhl, ale nevrátil nic (Null)."),
        }
        println!("\n✅ Hotovo.");
    }
}
