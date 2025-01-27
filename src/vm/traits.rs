use crate::vm::chunk::Constant;

// 演算用トレイト
pub trait ArithmeticOps {
    fn add(self, other: Self) -> Self;
    fn subtract(self, other: Self) -> Self;
    fn multiply(self, other: Self) -> Self;
    fn divide(self, other: Self) -> Self;
}

// 比較用トレイト
pub trait Comparable {
    fn equals(&self, other: f64) -> bool;
}

// Constant 型にトレイトを実装
impl ArithmeticOps for Constant {
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Constant::Number(a), Constant::Number(b)) => Constant::Number(a + b),
            _ => panic!("Addition is not supported for these Constant types"),
        }
    }

    fn subtract(self, other: Self) -> Self {
        match (self, other) {
            (Constant::Number(a), Constant::Number(b)) => Constant::Number(a - b),
            _ => panic!("Subtraction is not supported for these Constant types"),
        }
    }

    fn multiply(self, other: Self) -> Self {
        match (self, other) {
            (Constant::Number(a), Constant::Number(b)) => Constant::Number(a * b),
            _ => panic!("Multiplication is not supported for these Constant types"),
        }
    }

    fn divide(self, other: Self) -> Self {
        match (self, other) {
            (Constant::Number(a), Constant::Number(b)) => {
                if b == 0.0 {
                    panic!("Division by zero");
                }
                Constant::Number(a / b)
            }
            _ => panic!("Division is not supported for these Constant types"),
        }
    }
}

// 比較トレイトの実装
impl Comparable for Constant {
    fn equals(&self, other: f64) -> bool {
        match self {
            Constant::Number(n) => *n == other,
            _ => false,
        }
    }
}
