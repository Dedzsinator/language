use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    #[token("struct")]
    Struct,
    #[token("typeclass")]
    Typeclass,
    #[token("instance")]
    Instance,
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("Some")]
    Some,
    #[token("None")]
    None,
    #[token("in")]
    In,
    #[token("parallel")]
    Parallel,
    #[token("spawn")]
    Spawn,
    #[token("wait")]
    Wait,
    #[token("gpu")]
    Gpu,
    #[token("import")]
    Import,
    #[token("return")]
    Return,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,

    // Types
    #[token("Int")]
    IntType,
    #[token("Float")]
    FloatType,
    #[token("Bool")]
    BoolType,
    #[token("String")]
    StringType,
    #[token("Unit")]
    UnitType,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("^")]
    Caret,
    #[token("**")]
    DoubleStar,

    #[token("==")]
    EqualEqual,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,

    #[token("??")]
    QuestionQuestion, // Optional chaining operator

    #[token("@")]
    At, // Attribute marker

    // Assignment and arrows
    #[token("=")]
    Equal,
    #[token("=>")]
    Arrow,
    #[token("->")]
    ThinArrow,

    // Punctuation
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("?")]
    Question,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEqual,
    #[token("|")]
    Pipe,
    #[token("_")]
    Underscore,

    // Literals
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLiteral(i64),

    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string()) // Remove quotes
    })]
    StringLiteral(String),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Comments (skip)
    #[regex(r"--[^\r\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    Comment,

    // Whitespace (skip)
    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    Whitespace,
    // Error token (no longer needs #[error] attribute in logos 0.13)
    Error,

    // End of file
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Struct => write!(f, "struct"),
            Token::Typeclass => write!(f, "typeclass"),
            Token::Instance => write!(f, "instance"),
            Token::Let => write!(f, "let"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Match => write!(f, "match"),
            Token::Some => write!(f, "Some"),
            Token::None => write!(f, "None"),
            Token::In => write!(f, "in"),
            Token::Parallel => write!(f, "parallel"),
            Token::Spawn => write!(f, "spawn"),
            Token::Wait => write!(f, "wait"),
            Token::Gpu => write!(f, "gpu"),
            Token::Import => write!(f, "import"),
            Token::Return => write!(f, "return"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Null => write!(f, "null"),

            Token::IntType => write!(f, "Int"),
            Token::FloatType => write!(f, "Float"),
            Token::BoolType => write!(f, "Bool"),
            Token::StringType => write!(f, "String"),
            Token::UnitType => write!(f, "Unit"),

            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Caret => write!(f, "^"),
            Token::DoubleStar => write!(f, "**"),

            Token::EqualEqual => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),

            Token::AndAnd => write!(f, "&&"),
            Token::OrOr => write!(f, "||"),
            Token::Bang => write!(f, "!"),

            Token::QuestionQuestion => write!(f, "??"),
            Token::At => write!(f, "@"),

            Token::Equal => write!(f, "="),
            Token::Arrow => write!(f, "=>"),
            Token::ThinArrow => write!(f, "->"),

            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Question => write!(f, "?"),
            Token::Dot => write!(f, "."),
            Token::DotDot => write!(f, ".."),
            Token::DotDotEqual => write!(f, "..="),
            Token::Pipe => write!(f, "|"),
            Token::Underscore => write!(f, "_"),

            Token::IntLiteral(n) => write!(f, "{}", n),
            Token::FloatLiteral(n) => write!(f, "{}", n),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::Identifier(s) => write!(f, "{}", s),

            Token::Comment => write!(f, "comment"),
            Token::Whitespace => write!(f, "whitespace"),
            Token::Error => write!(f, "error"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

impl Token {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Struct
                | Token::Typeclass
                | Token::Instance
                | Token::Let
                | Token::If
                | Token::Else
                | Token::Match
                | Token::Some
                | Token::None
                | Token::In
                | Token::Parallel
                | Token::Spawn
                | Token::Wait
                | Token::Gpu
                | Token::Import
                | Token::Return
                | Token::True
                | Token::False
                | Token::Null
        )
    }

    pub fn is_type(&self) -> bool {
        matches!(
            self,
            Token::IntType
                | Token::FloatType
                | Token::BoolType
                | Token::StringType
                | Token::UnitType
        )
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::IntLiteral(_)
                | Token::FloatLiteral(_)
                | Token::StringLiteral(_)
                | Token::True
                | Token::False
        )
    }

    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::Caret
                | Token::DoubleStar
                | Token::EqualEqual
                | Token::NotEqual
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::GreaterEqual
                | Token::AndAnd
                | Token::OrOr
                | Token::Bang
                | Token::QuestionQuestion
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    #[test]
    fn test_keyword_tokens() {
        assert_eq!(Token::lexer("struct").next(), Some(Ok(Token::Struct)));
        assert_eq!(Token::lexer("typeclass").next(), Some(Ok(Token::Typeclass)));
        assert_eq!(Token::lexer("instance").next(), Some(Ok(Token::Instance)));
        assert_eq!(Token::lexer("let").next(), Some(Ok(Token::Let)));
        assert_eq!(Token::lexer("if").next(), Some(Ok(Token::If)));
        assert_eq!(Token::lexer("else").next(), Some(Ok(Token::Else)));
        assert_eq!(Token::lexer("match").next(), Some(Ok(Token::Match)));
        assert_eq!(Token::lexer("Some").next(), Some(Ok(Token::Some)));
        assert_eq!(Token::lexer("None").next(), Some(Ok(Token::None)));
        assert_eq!(Token::lexer("in").next(), Some(Ok(Token::In)));
        assert_eq!(Token::lexer("parallel").next(), Some(Ok(Token::Parallel)));
        assert_eq!(Token::lexer("spawn").next(), Some(Ok(Token::Spawn)));
        assert_eq!(Token::lexer("wait").next(), Some(Ok(Token::Wait)));
        assert_eq!(Token::lexer("gpu").next(), Some(Ok(Token::Gpu)));
        assert_eq!(Token::lexer("import").next(), Some(Ok(Token::Import)));
        assert_eq!(Token::lexer("return").next(), Some(Ok(Token::Return)));
        assert_eq!(Token::lexer("true").next(), Some(Ok(Token::True)));
        assert_eq!(Token::lexer("false").next(), Some(Ok(Token::False)));
        assert_eq!(Token::lexer("null").next(), Some(Ok(Token::Null)));
    }

    #[test]
    fn test_type_tokens() {
        assert_eq!(Token::lexer("Int").next(), Some(Ok(Token::IntType)));
        assert_eq!(Token::lexer("Float").next(), Some(Ok(Token::FloatType)));
        assert_eq!(Token::lexer("Bool").next(), Some(Ok(Token::BoolType)));
        assert_eq!(Token::lexer("String").next(), Some(Ok(Token::StringType)));
        assert_eq!(Token::lexer("Unit").next(), Some(Ok(Token::UnitType)));
    }

    #[test]
    fn test_operator_tokens() {
        assert_eq!(Token::lexer("+").next(), Some(Ok(Token::Plus)));
        assert_eq!(Token::lexer("-").next(), Some(Ok(Token::Minus)));
        assert_eq!(Token::lexer("*").next(), Some(Ok(Token::Star)));
        assert_eq!(Token::lexer("/").next(), Some(Ok(Token::Slash)));
        assert_eq!(Token::lexer("%").next(), Some(Ok(Token::Percent)));
        assert_eq!(Token::lexer("^").next(), Some(Ok(Token::Caret)));
        assert_eq!(Token::lexer("**").next(), Some(Ok(Token::DoubleStar)));
        assert_eq!(Token::lexer("==").next(), Some(Ok(Token::EqualEqual)));
        assert_eq!(Token::lexer("!=").next(), Some(Ok(Token::NotEqual)));
        assert_eq!(Token::lexer("<").next(), Some(Ok(Token::Less)));
        assert_eq!(Token::lexer("<=").next(), Some(Ok(Token::LessEqual)));
        assert_eq!(Token::lexer(">").next(), Some(Ok(Token::Greater)));
        assert_eq!(Token::lexer(">=").next(), Some(Ok(Token::GreaterEqual)));
        assert_eq!(Token::lexer("&&").next(), Some(Ok(Token::AndAnd)));
        assert_eq!(Token::lexer("||").next(), Some(Ok(Token::OrOr)));
        assert_eq!(Token::lexer("!").next(), Some(Ok(Token::Bang)));
        assert_eq!(Token::lexer("??").next(), Some(Ok(Token::QuestionQuestion)));
    }

    #[test]
    fn test_punctuation_tokens() {
        assert_eq!(Token::lexer("(").next(), Some(Ok(Token::LeftParen)));
        assert_eq!(Token::lexer(")").next(), Some(Ok(Token::RightParen)));
        assert_eq!(Token::lexer("[").next(), Some(Ok(Token::LeftBracket)));
        assert_eq!(Token::lexer("]").next(), Some(Ok(Token::RightBracket)));
        assert_eq!(Token::lexer("{").next(), Some(Ok(Token::LeftBrace)));
        assert_eq!(Token::lexer("}").next(), Some(Ok(Token::RightBrace)));
        assert_eq!(Token::lexer(",").next(), Some(Ok(Token::Comma)));
        assert_eq!(Token::lexer(";").next(), Some(Ok(Token::Semicolon)));
        assert_eq!(Token::lexer(".").next(), Some(Ok(Token::Dot)));
        assert_eq!(Token::lexer(":").next(), Some(Ok(Token::Colon)));
        assert_eq!(Token::lexer("=").next(), Some(Ok(Token::Equal)));
        assert_eq!(Token::lexer("->").next(), Some(Ok(Token::ThinArrow)));
        assert_eq!(Token::lexer("=>").next(), Some(Ok(Token::Arrow)));
        assert_eq!(Token::lexer("|").next(), Some(Ok(Token::Pipe)));
        assert_eq!(Token::lexer("_").next(), Some(Ok(Token::Underscore)));
        assert_eq!(Token::lexer("@").next(), Some(Ok(Token::At)));
        assert_eq!(Token::lexer("..").next(), Some(Ok(Token::DotDot)));
        assert_eq!(Token::lexer("..=").next(), Some(Ok(Token::DotDotEqual)));
    }

    #[test]
    fn test_literal_tokens() {
        // Test integer literals
        let mut lexer = Token::lexer("42");
        if let Some(Ok(Token::IntLiteral(value))) = lexer.next() {
            assert_eq!(value, 42);
        } else {
            panic!("Expected IntLiteral(42)");
        }

        // Test float literals
        let mut lexer = Token::lexer("3.14");
        if let Some(Ok(Token::FloatLiteral(value))) = lexer.next() {
            assert!((value - 3.14).abs() < f64::EPSILON);
        } else {
            panic!("Expected FloatLiteral(3.14)");
        }

        // Test string literals
        let mut lexer = Token::lexer("\"hello\"");
        if let Some(Ok(Token::StringLiteral(value))) = lexer.next() {
            assert_eq!(value, "hello");
        } else {
            panic!("Expected StringLiteral(\"hello\")");
        }

        // Test identifiers
        let mut lexer = Token::lexer("variable_name");
        if let Some(Ok(Token::Identifier(name))) = lexer.next() {
            assert_eq!(name, "variable_name");
        } else {
            panic!("Expected Identifier(\"variable_name\")");
        }
    }

    #[test]
    fn test_matrix_operators() {
        // These operators might not be implemented yet, so we test basic arithmetic
        assert_eq!(Token::lexer("*").next(), Some(Ok(Token::Star)));
        assert_eq!(Token::lexer("**").next(), Some(Ok(Token::DoubleStar)));
        assert_eq!(Token::lexer("^").next(), Some(Ok(Token::Caret)));
    }

    #[test]
    fn test_whitespace_and_comments() {
        // Whitespace should be skipped
        let mut lexer = Token::lexer("  \t\n  42");
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(42))));

        // Comments should be skipped (using -- for line comments)
        let mut lexer = Token::lexer("-- this is a comment\n42");
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(42))));

        let mut lexer = Token::lexer("/* block comment */ 42");
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(42))));
    }

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", Token::Plus), "+");
        assert_eq!(format!("{}", Token::Minus), "-");
        assert_eq!(format!("{}", Token::Star), "*");
        assert_eq!(format!("{}", Token::Let), "let");
        assert_eq!(format!("{}", Token::If), "if");
        assert_eq!(format!("{}", Token::IntType), "Int");
    }

    #[test]
    fn test_token_is_operator() {
        assert!(Token::Plus.is_operator());
        assert!(Token::Minus.is_operator());
        assert!(Token::Star.is_operator());
        assert!(Token::EqualEqual.is_operator());
        assert!(Token::Bang.is_operator());

        assert!(!Token::LeftParen.is_operator());
        assert!(!Token::Identifier("test".to_string()).is_operator());
        assert!(!Token::IntLiteral(42).is_operator());
    }

    #[test]
    fn test_multiple_tokens() {
        let input = "let x = 42 + 3.14";
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::Let)));
        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("x".to_string()))));
        assert_eq!(lexer.next(), Some(Ok(Token::Equal)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(42))));
        assert_eq!(lexer.next(), Some(Ok(Token::Plus)));
        assert_eq!(lexer.next(), Some(Ok(Token::FloatLiteral(3.14))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_complex_expression() {
        let input = "fn add(a: Int, b: Int) -> Int = a + b";
        let lexer = Token::lexer(input);

        let tokens: Vec<Token> = lexer.collect::<Result<Vec<_>, _>>().unwrap();

        assert!(tokens.len() > 10);
        assert_eq!(tokens[0], Token::Identifier("fn".to_string()));
        assert_eq!(tokens[1], Token::Identifier("add".to_string()));
        assert_eq!(tokens[2], Token::LeftParen);
    }

    #[test]
    fn test_matrix_literal() {
        let input = "[[1, 2], [3, 4]]";
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::LeftBracket)));
        assert_eq!(lexer.next(), Some(Ok(Token::LeftBracket)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::Comma)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(2))));
        assert_eq!(lexer.next(), Some(Ok(Token::RightBracket)));
        assert_eq!(lexer.next(), Some(Ok(Token::Comma)));
        assert_eq!(lexer.next(), Some(Ok(Token::LeftBracket)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(3))));
        assert_eq!(lexer.next(), Some(Ok(Token::Comma)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(4))));
        assert_eq!(lexer.next(), Some(Ok(Token::RightBracket)));
        assert_eq!(lexer.next(), Some(Ok(Token::RightBracket)));
    }

    #[test]
    fn test_string_with_escapes() {
        let input = r#""hello\nworld""#;
        let mut lexer = Token::lexer(input);

        if let Some(Ok(Token::StringLiteral(s))) = lexer.next() {
            assert_eq!(s, "hello\\nworld"); // The lexer should preserve the escape sequence
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_range_operators() {
        let input = "1..10";
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::DotDot)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(10))));

        let input = "1..=10";
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::DotDotEqual)));
        assert_eq!(lexer.next(), Some(Ok(Token::IntLiteral(10))));
    }

    #[test]
    fn test_error_handling() {
        // Test with invalid input
        let input = "§invalid_char";
        let mut lexer = Token::lexer(input);

        // Should produce an error for the invalid character
        match lexer.next() {
            Some(Err(_)) => (), // Expected
            _ => panic!("Expected lexer error for invalid character"),
        }
    }
}
