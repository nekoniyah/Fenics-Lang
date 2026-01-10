pub mod ast;
pub mod features;
pub mod interpreter;
pub mod interpreter_engine;
pub mod parser;
pub mod utils;

use std::env;
use tokio::fs;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: fenics-interpreter <file.fenics>");
        std::process::exit(1);
    }

    let filename = &args[1];

    let source = match fs::read_to_string(filename).await {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            std::process::exit(1);
        }
    };

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
