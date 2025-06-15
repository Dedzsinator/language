use matrix_lang::lexer::Lexer;

fn main() {
    let input = "        new_trail.remove(0)";
    println!("Debugging input: {}", input);

    let mut lexer = Lexer::new(input);

    loop {
        let token = lexer.next_token();
        println!("{:?}", token);
        if matches!(token.token, matrix_lang::lexer::Token::Eof) {
            break;
        }
    }
}
