use crate::lexer::Lexer;
use crate::parser::Parser;

fn main() {
    let source = "print(5)";
    println!("Attempting to parse: {}", source);
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).unwrap();
    
    match parser.parse_expression() {
        Ok(expr) => println!("Successfully parsed as expression: {:?}", expr),
        Err(e) => println!("Failed to parse as expression: {}", e),
    }
}
