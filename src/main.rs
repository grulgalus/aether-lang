mod lexer;
use lexer::Lexer;

fn main() {
    println!("Aether Compiler v0.1.0");
    
    let source_code = r#"
        actor Worker {
            fn start() {
                let x = "Hello"
            }
        }
    "#;

    println!("Compiling source code:\n{}", source_code);
    println!("--- Tokens ---");

    let mut lexer = Lexer::new(source_code);
    loop {
        let token = lexer.next_token();
        println!("{:?}", token);
        if token == lexer::Token::EOF {
            break;
        }
    }
}
