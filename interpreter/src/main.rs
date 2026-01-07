pub mod ast;
pub mod interpreter;
pub mod parser;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: fenics-interpreter <file.fenics>");
        std::process::exit(1);
    }

    let filename = &args[1];

    let source = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", filename, err);
        std::process::exit(1);
    });

    match parser::parse_program(&source) {
        Ok(program) => {
            let mut interpreter = interpreter::Interpreter::new();

            if let Err(err) = interpreter.interpret(&program) {
                eprintln!("Runtime error: {}", err);
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("Parse error: {}", err);
            std::process::exit(1);
        }
    }
}
