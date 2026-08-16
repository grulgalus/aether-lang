mod lexer;
use lexer::Lexer;

fn main() {
    println!("Aether Compiler v0.1.0");
    
    // Upravený kód, abychom otestovali i čísla a operátory!
    let source_code = r#"
        actor Worker {
            fn process(data: String) -> Result {
                let id = 42
                let ratio = 3.14
                if id != 0 {
                    return Ok("Done")
                }
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
