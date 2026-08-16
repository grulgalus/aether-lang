mod lexer;
mod ast;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    println!("Aether Compiler v0.1.0");
    
    let source_code = r#"
        let version = "1.0"
        let answer = 42
        let is_fast = 1
    "#;

    println!("Kompiluji zdrojový kód:\n{}", source_code);
    
    let lexer = Lexer::new(source_code);
    let mut parser = Parser::new(lexer);
    
    // Zde probíhá magie: Parser tvoří strom!
    let ast = parser.parse_program();
    
    println!("--- Abstraktní syntaktický strom (AST) ---");
    for stmt in ast.statements {
        println!("{:#?}", stmt);
    }
}
