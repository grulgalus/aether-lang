use std::env;
use std::fs;

mod lexer;
mod ast;
mod parser;
mod evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Použití: aether <soubor.ae>");
        std::process::exit(1);
    }
    
    let filename = &args[1];
    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Chyba: Nelze přečíst soubor '{}'", filename);
            std::process::exit(1);
        }
    };
        
    let lexer = lexer::Lexer::new(&contents);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.parse_program();
    
    let mut env = evaluator::Environment::new();
    evaluator::eval_program(&program, &mut env);
}
