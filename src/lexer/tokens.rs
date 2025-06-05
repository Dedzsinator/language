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
