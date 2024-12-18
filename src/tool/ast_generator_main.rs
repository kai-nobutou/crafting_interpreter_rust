use crafting_interpreter::tool::ast_generator;

fn main() {
    if let Err(e) = ast_generator::generate_ast("src/lox") {
        eprintln!("Failed to generate AST: {}", e);
    } else {
        println!("AST successfully generated!");
    }
}