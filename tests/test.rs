use crafting_interpreter::lox::error::LoxError;
use crafting_interpreter::lox::evaluator::{EvalResult, Evaluator}; // EvaluatorとEvalResultをインポート
use crafting_interpreter::lox::parser::Parser; // スクリプトパーサー
use crafting_interpreter::lox::scanner::Scanner;

#[cfg(test)]
mod tests {
    use std::fmt::{self, Debug};

    use super::*; // テスト対象のEvaluatorをインポート

    /// スクリプトを実行して結果を返すヘルパー関数
    fn run_script(input: &str) -> Result<String, LoxError> {
        let mut evaluator = Evaluator::new();

        // スキャナーでトークンを取得
        let tokens = Scanner::new(input).scan_tokens()?; // `LoxError` をそのまま返す

        // パーサーでステートメントを取得
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?; // `LoxError` をそのまま返す

        // ステートメントを評価
        match evaluator.evaluate_statements(statements) {
            EvalResult::Return(_) => Ok(evaluator.get_output()), // 正常終了時の結果を返す
            EvalResult::Error(err) => Err(err),                  // 評価中のエラーをそのまま返す
        }
    }

    #[test]
    fn test_literals() {
        let input = "print 123;";
        let expected_output = "123";
        let output = run_script(input);

        match output {
            Ok(actual_output) => assert_eq!(
                actual_output, expected_output,
                "Test failed for input: {}",
                input
            ),
            Err(err) => panic!("Test failed with error: {:?} for input: {}", err, input),
        }
    }

    #[test]
    fn test_variables() {
        let input = r#"
            var x = 10;
            print x;
        "#;
        let expected_output = "10";
        let output = run_script(input);

        match output {
            Ok(actual_output) => assert_eq!(
                actual_output, expected_output,
                "Test failed for input: {}",
                input
            ),
            Err(err) => panic!("Test failed with error: {:?} for input: {}", err, input),
        }
    }

    #[test]
    fn test_arithmetic() {
        let input = "print 1 + 2 * 3 / (4 - 1);";
        let expected_output = "3";
        let output = run_script(input);

        match output {
            Ok(actual_output) => assert_eq!(
                actual_output, expected_output,
                "Test failed for input: {}",
                input
            ),
            Err(err) => panic!("Test failed with error: {:?} for input: {}", err, input),
        }
    }

    #[test]
    fn test_conditionals() {
        let input = r#"
            if (true) print "yes";
            if (false) print "no"; else print "yes";
        "#;
        let expected_output = "yes\nyes";
        let output = run_script(input);

        match output {
            Ok(actual_output) => assert_eq!(
                actual_output, expected_output,
                "Test failed for input: {}",
                input
            ),
            Err(err) => panic!("Test failed with error: {:?} for input: {}", err, input),
        }
    }

    #[test]
    fn test_loops() {
        let input = r#"
            var i = 0;
            while (i < 3) {
                print i;
                i = i + 1;
            }
        "#;
        let expected_output = "0\n1\n2";
        let output = run_script(input);

        match output {
            Ok(actual_output) => assert_eq!(
                actual_output, expected_output,
                "Test failed for input: {}",
                input
            ),
            Err(err) => panic!("Test failed with error: {:?} for input: {}", err, input),
        }
    }

    #[test]
    fn test_functions() {
        let input = r#"
            fun add(a, b) {
                return a + b;
            }
            print add(3, 4);
        "#;
        let expected_output = "7";
        let output = run_script(input);

        match output {
            Ok(actual_output) => assert_eq!(
                actual_output, expected_output,
                "Test failed for input: {}",
                input
            ),
            Err(err) => panic!("Test failed with error: {:?} for input: {}", err, input),
        }
    }

    #[test]
    fn test_recursion() {
        let input = r#"
            fun factorial(n) {
                if (n <= 1) return 1;
                return n * factorial(n - 1);
            }
            print factorial(5);
        "#;
        let expected_output = "120";
        let output = run_script(input);

        match output {
            Ok(actual_output) => assert_eq!(
                actual_output, expected_output,
                "Test failed for input: {}",
                input
            ),
            Err(err) => panic!("Test failed with error: {:?} for input: {}", err, input),
        }
    }

    #[test]
    fn test_local_scope() {
        let input = r#"
            var a = "global";
            {
                var a = "local";
                print a;
            }
            print a;
        "#;
        let expected_output = "local\nglobal";
        let output = run_script(input);

        match output {
            Ok(actual_output) => assert_eq!(
                actual_output, expected_output,
                "Test failed for input: {}",
                input
            ),
            Err(err) => panic!("Test failed with error: {:?} for input: {}", err, input),
        }
    }

    #[test]
    fn test_error_messages() {
        let inputs = vec![
            ("print x;", "[Error: Undefined variable 'x']"), // Undefined variable
            (
                "if (123) print \"Invalid\";",
                "[Error: Non-boolean condition 'Condition must evaluate to a boolean.']",
            ), // Non-boolean condition
            ("var a = 10 / 0;", "[Error: Division by zero]"), // Division by zero
            (
                "fun f(a, a) { print a; }",
                "[Error: Parse error 'Duplicate parameter name 'a'.']",
            ), // Duplicate parameter names
            (
                "return 123;",
                "[Error: Cannot return from outside a function.]",
            ), // Return outside a function
        ];

        for (input, expected_error) in inputs {
            let output = run_script(input);
            match output {
                Ok(result) => {
                    print!("{}", result);
                    panic!("Expected error but got result: {}", result)
                }
                Err(err) => {
                    println!("{}", err); // エラーメッセージを表示
                    assert!(
                        format!("{}", err).contains(expected_error), // エラーメッセージに期待する文字列が含まれているか確認
                        "Test failed for input: {}\nExpected: {}\nGot: {}",
                        input,
                        expected_error,
                        err
                    );
                }
            }
        }
    }
}
