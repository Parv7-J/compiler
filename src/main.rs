#![allow(unused, dead_code)]
//TODO: Add spans -> Done
//DESIGN: Add unknown tokenkind -> Done
//OPTIONAL: Group Unknown tokens
//TODO: Add newline tracking -> Done
//TODO: Add comment support
//TODO: Add support for floatings
//TODO: Add support for negatives

//OPTIONAL REFACTOR: Instead of looping, delegate to eater functions

use compiler::*;
use std::fs;
use std::io::BufRead;

fn main() {
    // let mut stdin = std::io::stdin().lock();
    // let mut input = String::new();
    // stdin.read_line(&mut input).unwrap();

    let input = fs::read_to_string("language").unwrap();

    let lexer = Lexer::new(&input).unwrap();
    // let tokens = lexer.collect::<Vec<Token>>();
    // for token in tokens {
    //     println!(
    //         "TokenKind: {:?}, String: {}",
    //         token.kind,
    //         &input[token.span.start as usize..token.span.end as usize]
    //     );
    // }

    let parser = Parser::new(lexer);
    let ast = parser.parse();

    println!("AST: {ast:?}");
}
