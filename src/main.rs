#![allow(unused, dead_code)]
//TODO: Add spans -> Done
//DESIGN: Add unknown tokenkind -> Done
//OPTIONAL: Group Unknown tokens
//TODO: Add newline tracking -> Done -> remove as now handled by miette
//TODO: Add comment support
//TODO: Add support for floatings -> Done
//TODO: Add support for negatives -> - is unary, my bad

//OPTIONAL REFACTOR: Instead of looping, delegate to eater functions

use compiler::*;
use miette::MietteHandlerOpts;
use std::fs;
use std::io::BufRead;

fn main() -> miette::Result<()> {
    // let mut stdin = std::io::stdin().lock();
    // let mut input = String::new();
    // stdin.read_line(&mut input).unwrap();
    miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .context_lines(3) // <-- Change this to show more lines above and below
                .build(),
        )
    }))
    .unwrap();

    let fname = "custom";
    let input = fs::read_to_string(fname).unwrap();
    let lexer = Lexer::new(&input).unwrap();
    let tokens = lexer.clone().collect::<Vec<Token>>();
    for token in tokens {
        print!(
            "TokenKind: {:?}, String: {}  ",
            token.kind,
            &input[token.span.start as usize..token.span.end as usize]
        );
    }

    let parser = Parser::new(lexer);
    let ast = parser.parse(fname)?;
    println!("{ast:#?}");
    Ok(())
}
