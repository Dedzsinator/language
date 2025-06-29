#[cfg(test)]
mod debug_tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn debug_function_parsing() {
        let input = r#"
            let add = (a: Int, b: Int) -> Int => a + b
        "#;
        println!("Debugging input: {:?}", input);

        // Now try parsing
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer).expect("Failed to create parser");

        match parser.parse_program() {
            Ok(ast) => println!("Successfully parsed: {:?}", ast),
            Err(e) => {
                println!("Parse error: {}", e);
                panic!("Parse failed");
            }
        }
    }
}
