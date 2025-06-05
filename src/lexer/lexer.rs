use crate::lexer::tokens::Token;
use crate::ast::Span;
use logos::Logos;

/// A token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Span,
}

impl TokenWithSpan {
    pub fn new(token: Token, span: Span) -> Self {
        Self { token, span }
    }
}

/// Lexer for the matrix language
pub struct Lexer<'input> {
    input: &'input str,
    lexer: logos::Lexer<'input, Token>,
    line: usize,
    column: usize,
    last_pos: usize,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            lexer: Token::lexer(input),
            line: 1,
            column: 1,
            last_pos: 0,
        }
    }
    
    /// Get the next token with span information
    pub fn next_token(&mut self) -> TokenWithSpan {
        match self.lexer.next() {
            Some(Ok(token)) => {
                let span = self.current_span();
                self.update_position();
                TokenWithSpan::new(token, span)
            }
            Some(Err(_)) => {
                let span = self.current_span();
                self.update_position();
                TokenWithSpan::new(Token::Error, span)
            }
            None => {
                let span = Span::new(self.input.len(), self.input.len(), self.line, self.column);
                TokenWithSpan::new(Token::Eof, span)
            }
        }
    }
    
    /// Peek at the next token without consuming it
    pub fn peek_token(&self) -> Token {
        let mut cloned_lexer = self.lexer.clone();
        match cloned_lexer.next() {
            Some(Ok(token)) => token,
            Some(Err(_)) => Token::Error,
            None => Token::Eof,
        }
    }
    
    /// Get all tokens as a vector
    pub fn tokenize(mut self) -> Result<Vec<TokenWithSpan>, String> {
        let mut tokens = Vec::new();
        
        loop {
            let token_with_span = self.next_token();
            let is_eof = token_with_span.token == Token::Eof;
            
            if token_with_span.token == Token::Error {
                return Err(format!(
                    "Lexical error at line {}, column {}: unexpected character", 
                    token_with_span.span.line, 
                    token_with_span.span.column
                ));
            }
            
            tokens.push(token_with_span);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
    
    fn current_span(&self) -> Span {
        let range = self.lexer.span();
        Span::new(range.start, range.end, self.line, self.column)
    }
    
    fn update_position(&mut self) {
        let current_pos = self.lexer.span().end;
        let slice = &self.input[self.last_pos..current_pos];
        
        for ch in slice.chars() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        
        self.last_pos = current_pos;
    }
    
    /// Get the current position in the input
    pub fn position(&self) -> usize {
        self.lexer.span().start
    }
    
    /// Get the current line number
    pub fn line(&self) -> usize {
        self.line
    }
    
    /// Get the current column number
    pub fn column(&self) -> usize {
        self.column
    }
    
    /// Get the remaining input
    pub fn remaining(&self) -> &'input str {
        self.lexer.remainder()
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = TokenWithSpan;
    
    fn next(&mut self) -> Option<Self::Item> {
        let token_with_span = self.next_token();
        if token_with_span.token == Token::Eof {
            None
        } else {
            Some(token_with_span)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let input = "struct Vector2 { x: Float, y: Float }";
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::Struct);
        assert_eq!(tokens[1].token, Token::Identifier("Vector2".to_string()));
        assert_eq!(tokens[2].token, Token::LeftBrace);
        assert_eq!(tokens[3].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[4].token, Token::Colon);
        assert_eq!(tokens[5].token, Token::FloatType);
    }
    
    #[test]
    fn test_numbers() {
        let input = "42 3.14 1.5e-10";
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::IntLiteral(42));
        assert_eq!(tokens[1].token, Token::FloatLiteral(3.14));
        assert_eq!(tokens[2].token, Token::FloatLiteral(1.5e-10));
    }
    
    #[test]
    fn test_operators() {
        let input = "+ - * / % ^ ?? => -> == != <= >=";
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::Plus);
        assert_eq!(tokens[1].token, Token::Minus);
        assert_eq!(tokens[2].token, Token::Star);
        assert_eq!(tokens[3].token, Token::Slash);
        assert_eq!(tokens[4].token, Token::Percent);
        assert_eq!(tokens[5].token, Token::Caret);
        assert_eq!(tokens[6].token, Token::QuestionQuestion);
        assert_eq!(tokens[7].token, Token::Arrow);
        assert_eq!(tokens[8].token, Token::ThinArrow);
        assert_eq!(tokens[9].token, Token::EqualEqual);
        assert_eq!(tokens[10].token, Token::NotEqual);
        assert_eq!(tokens[11].token, Token::LessEqual);
        assert_eq!(tokens[12].token, Token::GreaterEqual);
    }
    
    #[test]
    fn test_string_literals() {
        let input = r#""hello world" "escaped \"quote\"" "multi\nline""#;
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token, Token::StringLiteral("hello world".to_string()));
        assert_eq!(tokens[1].token, Token::StringLiteral("escaped \\\"quote\\\"".to_string()));
        assert_eq!(tokens[2].token, Token::StringLiteral("multi\\nline".to_string()));
    }
    
    #[test]
    fn test_comments() {
        let input = r#"
            -- This is a line comment
            let x = 42
            /* This is a 
               block comment */
            let y = 3.14
        "#;
        let mut lexer = Lexer::new(input);
        
        let tokens: Vec<_> = lexer.tokenize().unwrap();
        
        // Comments should be skipped
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[2].token, Token::Equal);
        assert_eq!(tokens[3].token, Token::IntLiteral(42));
        assert_eq!(tokens[4].token, Token::Let);
        assert_eq!(tokens[5].token, Token::Identifier("y".to_string()));
        assert_eq!(tokens[6].token, Token::Equal);
        assert_eq!(tokens[7].token, Token::FloatLiteral(3.14));
    }
}
