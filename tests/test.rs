use crafting_interpreter::lox::evaluator::Evaluator; // Evaluatorをインポート
use crafting_interpreter::lox::evaluator::EvalResult; // EvalResultをインポート
use crafting_interpreter::lox::parser::Parser; // スクリプトパーサー
use crafting_interpreter::lox::scanner::Scanner;

#[cfg(test)]
mod tests {

    use super::*; // テスト対象のEvaluatorをインポート

    fn run_script(input: &str) -> String {
        let mut evaluator = Evaluator::new();
        let tokens = Scanner::new(input).scan_tokens();
        let mut parser = Parser::new(tokens);
        let statements = parser.parse();
        println!("{:?}", statements);
    
        evaluator.evaluate_statements(statements);
        evaluator.get_output() // 結果を取得
    }

    #[test]
    fn test_literals() {
        let input = "print 123;";
        let expected_output = "123";
        let output = run_script(input);
        assert_eq!(output, expected_output, "Test failed for input: {}", input);
    }

    #[test]
    fn test_variables() {
        let input = r#"
            var x = 10;
            print x;
        "#;
        let expected_output = "10";
        let output = run_script(input);
        assert_eq!(output, expected_output, "Test failed for input: {}", input);
    }

    #[test]
    fn test_arithmetic() {
        let input = "print 1 + 2 * 3 / (4 - 1);";
        let expected_output = "3";
        let output = run_script(input);
        assert_eq!(output, expected_output, "Test failed for input: {}", input);
    }

    #[test]
    fn test_conditionals() {
        let input = r#"
            if (true) print "yes";
            if (false) print "no"; else print "yes";
        "#;
        let expected_output = "yes\nyes";
        let output = run_script(input);
        assert_eq!(output, expected_output, "Test failed for input: {}", input);
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
        assert_eq!(output, expected_output, "Test failed for input: {}", input);
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
        assert_eq!(output, expected_output, "Test failed for input: {}", input);
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
        assert_eq!(output, expected_output, "Test failed for input: {}", input);
    }
}