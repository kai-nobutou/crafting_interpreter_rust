#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    // リテラル
    NumberLiteral(f64),
    StringLiteral(String),

    // 変数
    VariableDeclaration { name: String, initializer: Box<ASTNode> },
    VariableReference(String),

    // 演算子
    BinaryExpression {
        left: Box<ASTNode>,
        operator: BinaryOperator,
        right: Box<ASTNode>,
    },
    UnaryExpression {
        operator: UnaryOperator,
        right: Box<ASTNode>,
    },

    // グループ化
    Grouping(Box<ASTNode>),

    // 制御構文
    IfStatement {
        condition: Box<ASTNode>,
        then_branch: Vec<ASTNode>,
        else_branch: Option<Vec<ASTNode>>,
    },
    WhileStatement {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
    },

    // 関数
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<ASTNode>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<ASTNode>,
    },

    // ステートメント
    ExpressionStatement(Box<ASTNode>),
    ReturnStatement(Option<Box<ASTNode>>),

    // プログラムルート
    Program(Vec<ASTNode>),
}


#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
}