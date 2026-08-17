use std::env;
use std::fs;
use std::time::Instant;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::evaluator::{eval_program, Environment, Object};

mod lexer;
mod parser;
mod ast;
mod evaluator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Použití: aether <soubor.ae>");
        return;
    }

    if args[1] == "--studio" {
        println!("🚀 Spouštím Aether Studio na http://127.0.0.1:8765");
        return;
    }

    let start = Instant::now();
    let obsah = fs::read_to_string(&args[1]).expect("Nepodařilo se přečíst soubor");
    
    let lexer = Lexer::new(&obsah);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    
    // OPRAVA: Odstraněny argumenty (závorky jsou prázdné!)
    let mut env = Environment::new();
    
    let vysledek = eval_program(&program, &mut env);
    
    if let Object::Error(e) = vysledek {
        println!("{}", e);
    }
    
    println!("\n⏱️ Dokončeno za: {:.2?}", start.elapsed());
}
