use crate::vm::vm::Function;

#[derive(Debug)]
pub enum OpCode {
    OpConstant(u8),          // 定数をスタックにプッシュ
    OpAdd,                   // 足し算
    OpSubtract,              // 引き算
    OpMultiply,              // 掛け算
    OpDivide,                // 割り算
    OpNegate,                // 単項演算子 - (負の値を取る)
    OpReturn,                // 関数の終了
    OpJump(u16),             // 無条件ジャンプ
    OpJumpIfFalse(u16),      // 条件が偽の場合にジャンプ
    OpPop,                   // スタックのトップを破棄
    OpDefineGlobal(u8),      // グローバル変数を定義
    OpGetGlobal(u8),         // グローバル変数を取得
    OpSetGlobal(u8),         // グローバル変数を設定
    OpCall(u8, u8),          // 関数呼び出し (引数の数, 関数インデックス)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,  // バイトコード命令を格納
    pub constants: Vec<Constant>,  // 定数プール
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Number(f64),
    String(String),
    Function(Function),
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

// バイトコード命令の追加
    pub fn write_op(&mut self, op: OpCode) {
        match op {
            OpCode::OpConstant(index) => {
                self.code.push(0x01);           // OpConstantのオペコード（例：0x01）
                self.code.push(index as u8);   // インデックス
            }
            OpCode::OpAdd => {
                self.code.push(0x02);          // OpAddのオペコード
            }
            OpCode::OpSubtract => {
                self.code.push(0x03);          // OpSubtractのオペコード
            }
            OpCode::OpMultiply => {
                self.code.push(0x04);          // OpMultiplyのオペコード
            }
            OpCode::OpDivide => {
                self.code.push(0x05);          // OpDivideのオペコード
            }
            OpCode::OpNegate => {
                self.code.push(0x06);          // OpNegateのオペコード
            }
            OpCode::OpReturn => {
                self.code.push(0x07);          // OpReturnのオペコード
            }
            OpCode::OpJump(offset) => {
                self.code.push(0x08);          // OpJumpのオペコード
                self.code.push((offset & 0xFF) as u8);        // オフセットの下位バイト
                self.code.push(((offset >> 8) & 0xFF) as u8); // オフセットの上位バイト
            }
            OpCode::OpJumpIfFalse(offset) => {
                self.code.push(0x09);          // OpJumpIfFalseのオペコード
                self.code.push((offset & 0xFF) as u8);        // オフセットの下位バイト
                self.code.push(((offset >> 8) & 0xFF) as u8); // オフセットの上位バイト
            }
            OpCode::OpPop => {
                self.code.push(0x0A);          // OpPopのオペコード
            }
            OpCode::OpDefineGlobal(index) => {
                self.code.push(0x0B);          // OpDefineGlobalのオペコード
                self.code.push(index as u8);   // インデックス
            }
            OpCode::OpGetGlobal(index) => {
                self.code.push(0x0C);          // OpGetGlobalのオペコード
                self.code.push(index as u8);   // インデックス
            }
            OpCode::OpSetGlobal(index) => {
                self.code.push(0x0D);          // OpSetGlobalのオペコード
                self.code.push(index as u8);   // インデックス
            }
            OpCode::OpCall(arg_count, func_index) => {
                self.code.push(0x0E);          // OpCallのオペコード
                self.code.push(arg_count);     // 引数の数
                self.code.push(func_index);    // 関数のインデックス
            }
        }
    }

    // 定数の追加
    pub fn add_constant(&mut self, constant: Constant) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1 // インデックスを返す
    }

    pub fn write_jump(&mut self, op: OpCode) -> usize {
        self.write_op(op);
        self.code.len() - 1 
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump_distance = self.code.len() - offset - 1;
        self.code[offset] = jump_distance as u8; 
    }
}
