use std::env;
use std::path::PathBuf;
use std::fs;

fn main() {
    // initialize syntax/keywords/types/builtins registry
    hyperlight::syntax::register_defaults();

    let mut args = env::args().skip(1);
    let input = match args.next() {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!("Usage: hyperlight <source.hl>");
            std::process::exit(2);
        }
    };

    let src = match fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => { eprintln!("failed to read {}: {}", input.display(), e); std::process::exit(2); }
    };

    // Lex + parse
    match hyperlight::lexer::tokenize(&src) {
        Ok(toks) => {
            match hyperlight::parser::Parser::new(toks).parse() {
                Ok(stmts) => {
                    // typecheck
                    match hyperlight::typecheck::check(&stmts) {
                        Ok(_) => {
                            // compile and link native executable next to input (no extension)
                            let out = input.with_extension("");
                            match hyperlight::codegen::api::compile_and_link_executable(&stmts, &out) {
                                Ok(()) => println!("wrote executable to {}", out.display()),
                                Err(e) => { eprintln!("codegen error: {}", e); std::process::exit(1); }
                            }
                        }
                        Err(e) => { eprintln!("typecheck error: {:?}", e); std::process::exit(1); }
                    }
                }
                Err(e) => { eprintln!("parse error: {:?}", e); std::process::exit(1); }
            }
        }
        Err(e) => { eprintln!("lex error: {:?}", e); std::process::exit(1); }
    }
}
