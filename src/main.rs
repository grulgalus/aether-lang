use std::env;
use std::fs;

mod lexer;
mod ast;
mod parser;
mod evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();
    // Defaultní fallback na tests/test.ae, když nespustíme konkrétní soubor
    let filename = if args.len() > 1 { &args[1] } else { "tests/test.ae" };

    println!("🌌 Aether Compiler v0.1.0");
    println!("📂 Načítám soubor: {}", filename);
    
    let contents = fs::read_to_string(filename)
        .expect("Chyba: Nelze přečíst zadaný soubor!");
        
    println!("✨ Kód načten. Lexikální a syntaktická analýza...");
    
    let lexer = lexer::Lexer::new(&contents);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();
    
    println!("\n🚀 Spouštím virtuální stroj Aetheru...\n");
    
    let mut env = evaluator::Environment::new();
    let result = evaluator::eval_program(&program, &mut env);
    
    println!("--- VÝSLEDEK BĚHU PROGRAMU ---");
    // Tady je ta oprava! Přidali jsme podporu pro Object::Boolean(b)
    match result {
        evaluator::Object::Number(n) => println!("Vrácená hodnota: {}", n),
        evaluator::Object::StringObj(s) => println!("Vrácená hodnota: {}", s),
        evaluator::Object::Boolean(b) => println!("Vrácená hodnota: {}", b),
        evaluator::Object::Null => println!("Program proběhl, ale nevrátil nic (Null)."),
    }
    
    println!("\n✅ Hotovo.");
}
