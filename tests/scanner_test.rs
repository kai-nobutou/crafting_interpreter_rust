#[cfg(test)]
mod tests {
    use crafting_interpreter::lox::scanner::Scanner;
    use crafting_interpreter::lox::token_type::{LiteralValue, TokenType};

    #[test]
    fn test_scan_single_character_tokens() {
        let source = "(){},.-+;";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let expected_types = vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());

        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.token_type, expected_types[i]);
        }
    }

    #[test]
    fn test_scan_keywords_and_identifiers() {
        let source = "var x = 10; print x;";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let expected_types = vec![
            TokenType::Var,
            TokenType::Identifier,
            TokenType::Equal,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::Print,
            TokenType::Identifier,
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());

        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.token_type, expected_types[i]);
        }
    }

    #[test]
    fn test_scan_string_literal() {
        let source = "\"Hello, world!\"";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens.len(), 2); // StringLit + EOF
        assert_eq!(tokens[0].token_type, TokenType::StringLit);
        if let Some(LiteralValue::String(value)) = &tokens[0].literal {
            assert_eq!(value, "Hello, world!");
        } else {
            panic!("Expected string literal.");
        }
    }

    #[test]
    fn test_scan_number_literal() {
        let source = "123.45";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        assert_eq!(tokens.len(), 2); // Number + EOF
        assert_eq!(tokens[0].token_type, TokenType::Number);
        if let Some(LiteralValue::Number(value)) = &tokens[0].literal {
            assert_eq!(*value, 123.45);
        } else {
            panic!("Expected number literal.");
        }
    }

    #[test]
    fn test_scan_unterminated_string() {
        let source = "\"Hello, world!";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        // Expect an error message for unterminated string
        // Ensure EOF token is still present
        assert_eq!(tokens.len(), 1); // Only EOF
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn test_scan_comments() {
        let source = "// this is a comment\nvar x = 42;";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let expected_types = vec![
            TokenType::Var,
            TokenType::Identifier,
            TokenType::Equal,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::Eof,
        ];

        assert_eq!(tokens.len(), expected_types.len());

        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.token_type, expected_types[i]);
        }
    }

    #[test]
    fn test_unexpected_character() {
        let source = "#";
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        // Expect EOF token despite unexpected character
        assert_eq!(tokens.len(), 1); // Only EOF
        assert_eq!(tokens[0].token_type, TokenType::Eof);
    }
}
