use std::io::{self, Write};
use crafting_interpreter::vm::compiler::Compiler;
use crafting_interpreter::vm::vm::VM;
use crafting_interpreter::vm::ast_node::ASTNode;
use crafting_interpreter::vm::parser::{Parser, Token};

fn tokenize(input: String) -> Vec<Token> {
    input
        .split_whitespace()
        .map(|word| Token::Identifier(word.to_string())) // 適切にToken型を変換
        .collect()
}

fn main() {
    println!("Welcome to the LOX interpreter!");
    let mut input = String::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();

        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            continue;
        }

        let trimmed_input = input.trim();
        if trimmed_input.is_empty() || trimmed_input == "exit" {
            println!("Goodbye!");
            break;
        }

        // トークン化
        let tokens = tokenize(trimmed_input.to_string());

        // 解析 (パース)
        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(err) => {
                eprintln!("Parse error: {}", err);
                continue;
            }
        };

        // コンパイル
        let mut compiler = Compiler::new();
        let chunk = match compiler.compile(&ast) {
            Ok(chunk) => chunk,
            Err(err) => {
                eprintln!("Compilation error: {}", err);
                continue;
            }
        };

        // 実行
        let mut vm = VM::new(chunk.clone());
        vm.execute(); // 結果を直接処理
        println!("Execution complete. Stack: {:?}", vm.stack);
    }
}