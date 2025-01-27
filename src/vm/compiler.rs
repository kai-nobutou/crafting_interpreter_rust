use crate::vm::chunk::{Chunk, Constant, OpCode};
use crate::vm::ast_node::{ASTNode, BinaryOperator, UnaryOperator};
use crate::vm::vm::Function;

pub struct Compiler {
    chunk: Chunk, 
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
        }
    }

    pub fn compile(&mut self, node: &ASTNode) -> Result<&Chunk, String> {
        self.compile_node(node)?;
        self.chunk.write_op(OpCode::OpReturn); 
        Ok(&self.chunk)
    }

    fn compile_node(&mut self, node: &ASTNode) -> Result<(), String> {
        match node {

            // 二項演算子
            ASTNode::BinaryExpression { left, operator, right } => {
                self.compile_node(left)?;
                self.compile_node(right)?; 

                match operator {
                    BinaryOperator::Plus => self.chunk.write_op(OpCode::OpAdd),
                    BinaryOperator::Minus => self.chunk.write_op(OpCode::OpSubtract),
                    BinaryOperator::Star => self.chunk.write_op(OpCode::OpMultiply),
                    BinaryOperator::Slash => self.chunk.write_op(OpCode::OpDivide),
                    _ => return Err(format!("Unsupported binary operator: {:?}", operator)),
                }
            }

            // 単項演算子
            ASTNode::UnaryExpression { operator, right } => {
                self.compile_node(right)?; 
                match operator {
                    UnaryOperator::Minus => self.chunk.write_op(OpCode::OpNegate),
                    _ => return Err(format!("Unsupported unary operator: {:?}", operator)),
                }
            }

            // 変数宣言
            ASTNode::VariableDeclaration { name, initializer } => {
                self.compile_node(initializer)?; 
                let index = self.chunk.add_constant(Constant::String(name.clone()));
                self.chunk.write_op(OpCode::OpDefineGlobal(index.try_into().expect("Index too large for u8")));
            }

            // 変数参照
            ASTNode::VariableReference(name) => {
                let index = self.chunk.add_constant(Constant::String(name.clone()));
                self.chunk.write_op(OpCode::OpDefineGlobal(index.try_into().expect("Index too large for u8")));
            }

            // 条件式（if文）
            ASTNode::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_node(condition)?; 

                let jump_if_false = self.chunk.write_jump(OpCode::OpJumpIfFalse(0)); 
                self.chunk.write_op(OpCode::OpPop); 

                for statement in then_branch {
                    self.compile_node(statement)?;
                }

                let jump_to_end = self.chunk.write_jump(OpCode::OpJump(0)); 

                self.chunk.patch_jump(jump_if_false); 

                if let Some(else_branch) = else_branch {
                    for statement in else_branch {
                        self.compile_node(statement)?;
                    }
                }

                self.chunk.patch_jump(jump_to_end); 
            }

            // 関数宣言
            ASTNode::FunctionDeclaration {
                name,
                parameters,
                body,
            } => {
                let function_chunk = Chunk::new();

                let mut function_compiler = Compiler { chunk: function_chunk };
                for statement in body {
                    function_compiler.compile_node(statement)?;
                }

                function_compiler.chunk.write_op(OpCode::OpReturn);

                let function_index = self.chunk.add_constant(Constant::Function(Function {
                    name: name.clone(),
                    chunk: function_compiler.chunk,
                    arity: parameters.len(),
                }));
                self.chunk.write_op(OpCode::OpConstant(function_index.try_into().expect("Index too large for u8")));
            }

            // 数値リテラル
            ASTNode::NumberLiteral(value) => {
                let index = self.chunk.add_constant(Constant::Number(*value));
                self.chunk.write_op(OpCode::OpConstant(index.try_into().expect("Index too large for u8")));
            }

            // 関数呼び出し
            ASTNode::FunctionCall { name, arguments } => {
                for arg in arguments {
                    self.compile_node(arg)?; 
                }
                let index = self.chunk.add_constant(Constant::String(name.clone()));
                self.chunk.write_op(OpCode::OpCall(arguments.len() as u8, index as u8));
            }


            ASTNode::ReturnStatement(value) => {
                if let Some(expr) = value {
                    self.compile_node(expr)?;
                } else {
                    let constant_index = self.chunk.add_constant(Constant::Number(0.0));
                    self.chunk.write_op(OpCode::OpConstant(
                        constant_index.try_into().expect("Index too large for u8"),
                    ));
                }
                self.chunk.write_op(OpCode::OpReturn);
            }

            _ => return Err(format!("Unsupported AST node: {:?}", node)),
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::chunk::{Chunk, OpCode, Constant};
    use crate::vm::ast_node::{ASTNode, BinaryOperator, UnaryOperator};

    #[test]
    fn test_compile_number_literal() {
        let mut compiler = Compiler::new();
        let ast = ASTNode::NumberLiteral(42.0);

        let chunk = compiler.compile(&ast).expect("Failed to compile");
        assert_eq!(chunk.code, vec![0x01, 0x00, 0x07]); // OpConstant + index 0 + OpReturn
        assert_eq!(chunk.constants, vec![Constant::Number(42.0)]);
    }

    #[test]
    fn test_compile_binary_expression() {
        let mut compiler = Compiler::new();
        let ast = ASTNode::BinaryExpression {
            left: Box::new(ASTNode::NumberLiteral(1.0)),
            operator: BinaryOperator::Plus,
            right: Box::new(ASTNode::NumberLiteral(2.0)),
        };

        let chunk = compiler.compile(&ast).expect("Failed to compile");
        assert_eq!(
            chunk.code,
            vec![0x01, 0x00, 0x01, 0x01, 0x02, 0x07] // OpConstant(1) + OpConstant(2) + OpAdd + OpReturn
        );
        assert_eq!(
            chunk.constants,
            vec![Constant::Number(1.0), Constant::Number(2.0)]
        );
    }

    #[test]
    fn test_compile_variable_declaration() {
        let mut compiler = Compiler::new();
        let ast = ASTNode::VariableDeclaration {
            name: "x".to_string(),
            initializer: Box::new(ASTNode::NumberLiteral(10.0)),
        };

        let chunk = compiler.compile(&ast).expect("Failed to compile");
        assert_eq!(
            chunk.code,
            vec![0x01, 0x00, 0x0B, 0x01, 0x07] // OpConstant(10) + OpDefineGlobal("x") + OpReturn
        );
        assert_eq!(
            chunk.constants,
            vec![Constant::Number(10.0), Constant::String("x".to_string())]
        );
    }

    #[test]
    fn test_compile_if_statement() {
        let mut compiler = Compiler::new();
        let ast = ASTNode::IfStatement {
            condition: Box::new(ASTNode::NumberLiteral(1.0)),
            then_branch: vec![ASTNode::NumberLiteral(2.0)],
            else_branch: Some(vec![ASTNode::NumberLiteral(3.0)]),
        };

        let chunk = compiler.compile(&ast).expect("Failed to compile");

        // 条件 + ジャンプ命令の確認
        assert!(chunk.code.contains(&0x09)); // OpJumpIfFalse
        assert!(chunk.code.contains(&0x08)); // OpJump
        assert_eq!(
            chunk.constants,
            vec![
                Constant::Number(1.0),
                Constant::Number(2.0),
                Constant::Number(3.0)
            ]
        );
    }
}