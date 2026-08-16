mod lexer;
mod ast;
mod parser;

use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;

fn main() {
    // Načtení argumentů z příkazové řádky
    let args: Vec<String> = env::args().collect();
    
    // Pokud uživatel nezadá soubor, použijeme výchozí "test.ae"
    let filename = if args.len() > 1 {
        &args[1]
    } else {
        "test.ae"
    };

    println!("🌌 Aether Compiler v0.1.0");
    println!("📂 Načítám soubor: {}", filename);

    // Pokus o načtení obsahu souboru
    let source_code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Chyba při čtení souboru '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    println!("✨ Kód úspěšně načten. Zahajuji lexikální a syntaktickou analýzu...\n");
    
    let lexer = Lexer::new(&source_code);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program();
    
    println!("--- Abstraktní syntaktický strom (AST) ---");
    for stmt in ast.statements {
        println!("{:#?}", stmt);
    }
    
    println!("\n✅ Analýza dokončena. Nalezeno {} hlavních uzlů.", ast.statements.len());
}
