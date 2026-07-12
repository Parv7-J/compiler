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

    let lexer = Lexer::new(&input);

    let ts = TokenStream::new(lexer.unwrap());
    println!("{ts:#?}");

    let mut parser = Parser::new(ts, &input);
    let ast = parser.parse();

    println!("{ast:#?}");
}
