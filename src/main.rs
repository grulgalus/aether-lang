mod lexer;
mod ast;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    println!("Aether Compiler v0.1.0");
    
    // Nyní již kompilátor rozumí plnohodnotným funkcím a blokům!
    let source_code = r#"
        fn start_engine() {
            let status = "Running"
            let power = 100
            return status
        }
    "#;

    println!("Kompiluji zdrojový kód:\n{}", source_code);
    
    let lexer = Lexer::new(source_code);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program();
    
    println!("--- Abstraktní syntaktický strom (AST) ---");
    for stmt in ast.statements {
        println!("{:#?}", stmt);
    }
}
