mod lexer;
mod ast;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    println!("Aether Compiler v0.1.0");
    
    // Nyní náš kompilátor dokáže rozložit celý koncept Aetheru!
    let source_code = r#"
        actor DataMiner {
            fn extract() {
                let status = "mining"
                return status
            }
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
