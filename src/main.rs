mod lexer;
mod syntax;

use crate::lexer::{Lexer, Rule};
use crate::syntax::Syntax;
use pest::Parser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pest::set_error_detail(false);
    let file_content = fs::read_to_string("assets/helloworld.aoc")?;
    let parsed = Lexer::parse(Rule::program, &file_content)
        .unwrap()
        .next()
        .unwrap()
        .into_inner();

    let mut syntax = Syntax::new();
    syntax.analize(parsed.clone());
    syntax.optimize();
    syntax.assemble(parsed);

    //let mut assembler = Assembler::new(parsed.tokens());
    //assembler.assemble();

    Ok(())
}
