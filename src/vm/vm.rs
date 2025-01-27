use crate::vm::chunk::{Chunk, Constant};
use crate::vm::traits::ArithmeticOps;
use core::panic;
use std::collections::HashMap;
use std::fmt::{self};
use std::cmp::PartialEq;

pub struct VM {
    pub ip: usize,           // 命令ポインタ（Instruction Pointer）
    pub chunk: Chunk,        // 実行するチャンク
    pub stack: Vec<Constant>, // スタック
    pub global_table: GlobalTable, // グローバル変数管理
    pub frames: Vec<CallFrame>, 
}

impl VM {
    pub fn new(chunk: Chunk) -> VM {
        VM {
            ip: 0,
            chunk,
            stack: Vec::new(),
            global_table: GlobalTable::new(),
            frames: Vec::new()
        }
    }

    // バイトコードを実行
    pub fn execute(&mut self) {
        let mut ip = 0; // 命令ポインタ
        while ip < self.chunk.code.len() {
            let opcode = self.chunk.code[ip] as u8;
            ip += 1;

            match opcode {
                0x01 => {
                    self.op_constant(self.chunk.code[ip] as usize); // OpConstant
                    ip += 1; // 次の命令に進む
                }
                0x02 => self.op_add(),         // OpAdd
                0x03 => self.op_subtract(),    // OpSubtract
                0x04 => self.op_multiply(),    // OpMultiply
                0x05 => self.op_divide(),      // OpDivide
                0x06 => self.op_return(),      // OpReturn
                0x07 => {
                    let offset = self.chunk.code[ip] as usize;
                    self.op_jump(offset);
                }
                0x08 => {
                    let offset = self.chunk.code[ip] as usize;
                    ip += 1;
                    self.op_jump_false(offset);
                }
                0x09 => {
                    let index = self.chunk.code[ip] as usize;
                    ip += 1;
                    let value = self.stack.pop().expect("Stack underflow");
                    self.op_define_global(index, value);
                }
                0x0A => {
                    let index = self.chunk.code[ip] as usize;
                    ip += 1;
                    self.op_get_global(index);
                }
                0x0B => {
                    let index = self.chunk.code[ip] as usize;
                    ip += 1;
                    self.op_set_global(index);
                }
                0x0C => {
                    let argument_count = self.chunk.code[ip] as usize;
                    ip += 1;
                    self.op_call(argument_count);
                }
                _ => panic!("Unknown OpCode: {}", opcode),
            }
        }
    }

    fn op_constant(&mut self, index: usize) {
        if let Some(constant) = self.chunk.constants.get(index) {
            match constant {
                Constant::Number(n) => self.stack.push(Constant::Number(*n)),
                _ => {}
            }
        }
    }

    fn op_add(&mut self) {
        let b = self.stack.pop().expect("Stack underflow");
        let a = self.stack.pop().expect("Stack underflow");
        self.stack.push(a.add(b));
    }

    fn op_subtract(&mut self) {
        let b = self.stack.pop().expect("Stack underflow");
        let a = self.stack.pop().expect("Stack underflow");
        self.stack.push(a.subtract(b));
    }

    fn op_multiply(&mut self) {
        let b = self.stack.pop().expect("Stack underflow");
        let a = self.stack.pop().expect("Stack underflow");
        self.stack.push(a.multiply(b));
    }

    fn op_divide(&mut self) {
        let b = self.stack.pop().expect("Stack underflow");
        let a = self.stack.pop().expect("Stack underflow");
        self.stack.push(a.divide(b));
    }

    fn op_return(&mut self) {
        // 関数の戻り値を取得
        let value = self.stack.pop().expect("Stack underflow");

        // フレームを終了
        let frame = self.frames.pop().expect("Call frame underflow");

        // スタックを元に戻す
        self.stack.truncate(frame.base_pointer);

        // 戻り値をプッシュ
        self.stack.push(value);
    }

    // OpCode: OpJumpz
    fn op_jump(&mut self, offset: usize) {
        self.ip = offset; // 指定された位置にジャンプ
    }

    // OpCode: OpJumpFalse
    fn op_jump_false(&mut self, offset: usize) {
        let condition = self.stack.pop().expect("Stack underflow");
        if condition == 0.0 {
            self.ip = offset; // 条件が偽ならジャンプ
        }
    }

    fn op_define_global(&mut self, index: usize, value: Constant) {
        if let Err(err) = self.global_table.define(index, value) {
            panic!("{}", err); // 定義済みの場合にパニック
        }
    }

    // OpCode: OpGetGlobal
    fn op_get_global(&mut self, index: usize) {
        match self.global_table.get(index) {
            Some(value) => self.stack.push(value.clone()), // クローンしてスタックにプッシュ
            None => panic!("Undefined global variable at index {}", index),
        }
    }

    // OpCode: OpSetGlobal
    fn op_set_global(&mut self, index: usize) {
        if let Some(value) = self.stack.pop() {
            self.global_table.set(index, value); // グローバル変数を更新
        } else {
            panic!("Stack underflow while setting global variable");
        }
    }

    // OpCode: OpCall
    fn op_call(&mut self, argument_count: usize) {
        // スタックから関数を取得
        let function = match self.stack.pop() {
            Some(Constant::Function(func)) => func,
            _ => panic!("Expected a function on the stack"),
        };

        // 引数の数を確認
        if function.arity != argument_count {
            panic!(
                "Expected {} arguments but got {}",
                function.arity, argument_count
            );
        }

        // 新しいフレームを作成
        let base_pointer = self.stack.len() - argument_count;
        let frame = CallFrame::new(function, base_pointer);

        // フレームを追加
        self.frames.push(frame);

        // IP を初期化
        self.ip = 0;
    }
}




///
/// コンスタント（定数）を表示する際フォーマットを適用する
/// 
/// ---
/// print!("{}", constant);
impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Number(n) => write!(f, "{}", n),
            Constant::String(s) => write!(f, "{}", s),
            Constant::Function(func) => write!(f, "{}", func.name),
        }
    }
}



///
/// Constant::Number と f64 を比較できるようにする関数
/// 
impl PartialEq<f64> for Constant {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Constant::Number(n) => n == other,
            _ => false, // 他の型は等しくないとみなす
        }
    }
}


///
/// グローバル変数を管理する
/// 
pub struct GlobalTable {
    globals: HashMap<usize, Constant>, // グローバル変数を格納
}

impl GlobalTable {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Constant> {
        self.globals.get(&index)
    }

    pub fn set(&mut self, index: usize, value: Constant) {
        self.globals.insert(index, value);
    }

    pub fn define(&mut self, index: usize, value: Constant) -> Result<(), &'static str> {
        if self.globals.contains_key(&index) {
            Err("Global variable already defined")
        } else {
            self.globals.insert(index, value);
            Ok(())
        }
    }
}

/// 関数の情報を保持する構造体

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub chunk: Chunk,
    pub arity: usize,
}

impl Function {
    pub fn new(name: &str, chunk: Chunk, arity: usize) -> Self {
        Self {
            name: name.to_string(),
            chunk,
            arity,
        }
    }
}

/// 関数呼び出し時の情報を管理する
pub struct CallFrame {
    pub function: Function,  // 呼び出された関数
    pub ip: usize,           // 関数内の命令ポインタ
    pub base_pointer: usize, // スタックの基準位置
}

impl CallFrame {
    pub fn new(function: Function, base_pointer: usize) -> Self {
        Self {
            function,
            ip: 0,
            base_pointer,
        }
    }
}
