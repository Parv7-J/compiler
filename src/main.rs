use compiler::*;
use miette::MietteHandlerOpts;
use std::fs;

fn main() -> miette::Result<()> {
    // let mut stdin = std::io::stdin().lock();
    // let mut input = String::new();
    // stdin.read_line(&mut input).unwrap();

    miette::set_hook(Box::new(|_| {
        Box::new(MietteHandlerOpts::new().context_lines(3).build())
    }))
    .unwrap();

    let mut args = std::env::args();
    //auto panics if not valid unicode, so fname is always a string only
    args.next();
    //TODO: introduce clap
    let fname = args.next().unwrap_or(String::from("language"));

    let input = fs::read_to_string(&fname).unwrap();
    let lexer = Lexer::new(&input).unwrap();

    // let tokens = lexer.clone().collect::<Vec<Token>>();
    // for token in tokens {
    //     print!(
    //         "TokenKind: {:?}, String: {}  ",
    //         token.kind,
    //         &input[token.span.start as usize..token.span.end as usize]
    //     );
    // }

    let parser = Parser::new(lexer);
    let ast = parser.parse(&fname);
    println!("{:#?}", ast);
    //
    // let analyzer = AstAnalyzer::new(ast);
    // let output = analyzer.analyze();
    //
    // println!("{output:#?}");
    //
    Ok(())
}
