use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub fn generate_ast(output_dir: &str) -> io::Result<()> {
    let file_path = Path::new(output_dir).join("ast.rs");
    let mut file = File::create(&file_path)?;

    writeln!(file, "use crate::lox::token::Token;")?;
    writeln!(file, "use crate::lox::token_type::LiteralValue;")?;
    writeln!(file, "\n#[derive(Debug)]")?;
    writeln!(file, "pub enum Expr {{")?;

    let node_types = vec![
        ("Binary", "left: Box<Expr>, operator: Token, right: Box<Expr>"),
        ("Grouping", "expression: Box<Expr>"),
        ("Literal", "value: LiteralValue"),
        ("Unary", "operator: Token, operand: Box<Expr>"),
    ];

    for (class_name, fields) in node_types {
        writeln!(file, "    {} {{ {} }},", class_name, fields)?;
    }

    writeln!(file, "}}\n")?;

    writeln!(file, "pub enum Stmt {{")?;

    let stmt_types = vec![
        ("Expression", "Expr"),
        ("Print", "Expr"),
    ];

    for (clss_name,fields) in stmt_types {
        writeln!(file, "    {} {{ {} }},", clss_name, fields)?;
    }
    
    writeln!(file, "}}\n")?;

    Ok(())
}