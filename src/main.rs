use std::env;
use std::fs;
use std::io::{self, BufRead, Write};

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
    
    let mut parser = lox::parser::Parser::new(tokens);
    let statements = parser.parse();

    let mut evaluator = lox::evaluator::Evaluator::new();
    evaluator.evaluate_statements(statements);
}