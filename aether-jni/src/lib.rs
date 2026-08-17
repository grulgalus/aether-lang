use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

// Natáhneme si přímo tvůj kompilátor!
#[path = "../../src/lexer.rs"] mod lexer;
#[path = "../../src/ast.rs"] mod ast;
#[path = "../../src/parser.rs"] mod parser;
#[path = "../../src/evaluator.rs"] mod evaluator;

// Tahle funkce bude viditelná pro Javu!
#[no_mangle]
pub extern "system" fn Java_com_aether_studio_AetherCore_runCode<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    input: JString<'local>,
) -> jstring {
    let input_str: String = env.get_string(&input).expect("Chyba čtení stringu").into();
    
    let lex = lexer::Lexer::new(&input_str);
    let mut par = parser::Parser::new(lex);
    let prog = par.parse_program();
    let mut aether_env = evaluator::Environment::new(false);
    
    evaluator::eval_program(&prog, &mut aether_env);
    
    let mut out = aether_env.output.join("\n");
    if out.is_empty() { out = "OK (Bez výstupu)".to_string(); }

    let output = env.new_string(out).expect("Chyba tvorby stringu");
    output.into_raw()
}
