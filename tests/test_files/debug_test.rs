use matrix_lang::lexer::Lexer;

fn main() {
    let input = "let add = (a: Int, b: Int) -> Int => a + b";
    println!("Debugging input: {}", input);

    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        match lexer.next_token() {
            Ok(token) => {
                println!("{:?}", token);
                if matches!(token.token, matrix_lang::lexer::tokens::Token::Eof) {
                    break;
                }
                tokens.push(token);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }

    // Try parsing
    let lexer2 = Lexer::new(input);
    let mut parser = matrix_lang::parser::Parser::new(lexer2).expect("Failed to create parser");
    match parser.parse_program() {
        Ok(ast) => println!("Successfully parsed: {:?}", ast),
        Err(e) => println!("Parse error: {}", e),
    }
}
