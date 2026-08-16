mod lexer;
mod ast;
mod parser;
mod evaluator;

use lexer::Lexer;
use parser::Parser;
use evaluator::{Environment, eval_program};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "test.ae"
    };

    println!("🌌 Aether Compiler v0.1.0");
    println!("📂 Načítám soubor: {}", filename);

    let source_code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Chyba při čtení souboru '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    println!("✨ Kód načten. Lexikální a syntaktická analýza...\n");
    
    let lexer = Lexer::new(&source_code);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program();
    
    // Nyní přidáme spuštění kódu (Evaluator)!
    println!("🚀 Spouštím virtuální stroj Aetheru...\n");
    let mut env = Environment::new();
    let result = eval_program(&ast, &mut env);

    println!("--- VÝSLEDEK BĚHU PROGRAMU ---");
    match result {
        evaluator::Object::Number(n) => println!("Vrácená hodnota: {}", n),
        evaluator::Object::StringObj(s) => println!("Vrácená hodnota: \"{}\"", s),
        evaluator::Object::Null => println!("Program proběhl, ale nevrátil nic (Null)."),
    }
    
    println!("\n✅ Hotovo.");
}
