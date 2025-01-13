use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use crate::lox::evaluator::EvalResult;

mod lox;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: rlox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }
}

fn run_file(path: &str) {
    println!("Running file at path: {}", path); 
    let source = fs::read_to_string(path).expect("Failed to read the file");
    run(&source);
}

fn run_prompt() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        write!(stdout, "> ").unwrap();
        stdout.flush().unwrap();

        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line).unwrap();

        if bytes_read == 0 {
            break;
        }

        run(&line);
    }
}

fn run(source: &str) {

    let mut scanner = lox::scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens();
   
    if tokens.is_empty() {
        eprintln!("No tokens found, check scanner implementation.");
        return;
    }

    let mut parser = lox::parser::Parser::new(tokens);
    let statements = parser.parse();

    if statements.is_empty() {
        eprintln!("Parse error: No valid statements produced.");
        return;
    }

    let mut evaluator = lox::evaluator::Evaluator::new();
    match evaluator.evaluate_statements(statements) {
        EvalResult::Return(_) => println!("Evaluation completed successfully."),
        EvalResult::Error(e) => eprintln!("Evaluation error: {}", e),
    }
}