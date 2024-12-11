use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process;

mod lox;

struct ErrorReporter {
    had_error: bool,
}

impl ErrorReporter {
    fn new() -> Self {
        ErrorReporter { had_error: false }
    }

    fn error(&mut self, line: usize, message: &str) {
        eprintln!("[line {}] Error: {}", line, message);
        self.had_error = true;
    }

    fn reset(&mut self) {
        self.had_error = false;
    }

    fn had_error(&self) -> bool {
        self.had_error
    }
}

fn main() -> io::Result<()> {
    let mut reporter = ErrorReporter::new();
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            run_prompt(&mut reporter)?;
        }
        2 => {
            run_file(&args[1], &mut reporter)?;
        }
        _ => {
            eprintln!("Usage: rlox [script]");
            process::exit(64);
        }
    }

    if reporter.had_error() {
        process::exit(65);
    }

    Ok(())
}

fn run_file(path: &str, reporter: &mut ErrorReporter) -> io::Result<()> {
    let source = fs::read_to_string(path)?;
    run(&source, reporter);
    if reporter.had_error() {
        process::exit(65);
    }
    Ok(())
}

fn run_prompt(reporter: &mut ErrorReporter) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line)?;

        if bytes_read == 0 {
            break;
        }

        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }

        run(line, reporter);
        reporter.reset();
    }

    Ok(())
}

fn run(source: &str, reporter: &mut ErrorReporter) {
    if source.contains("error") {
        reporter.error(1, "Found 'error' in input.");
    } else {
        println!("Running: {}", source);
    }
}