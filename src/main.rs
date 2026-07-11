#![allow(unused, dead_code)]
//TODO: Add spans
//DESIGN: Add unknown tokenkind
//OPTIONAL: Group Unknown tokens
//TODO: Add newline tracking
//TODO: Add comment support
//TODO: Add support for floatings
//TODO: Add support for negatives

//OPTIONAL REFACTOR: Instead of looping, delegate to eater functions

use compiler::*;
use std::fs;
// use std::io::BufRead;

fn main() {
    // let mut stdin = std::io::stdin().lock();
    // let mut input = String::new();
    // stdin.read_line(&mut input).unwrap();

    let input = fs::read_to_string("language").unwrap();

    let lexer = Lexer::new(&input);

    let ts = TokenStream::new(lexer);
    println!("{ts:?}");

    let mut parser = Parser::new(&ts);
    let ast = parser.parse();
}
