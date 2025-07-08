mod assembler;
mod lexer;
mod syntax;

use crate::assembler::Assembler;
use crate::lexer::{Lexer, Rule};
use crate::syntax::Syntax;
use pest::Parser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("assets/helloworld.aoc")?;

    let mut parse_result = Lexer::parse(Rule::program, &file_content).map_err(|e| format!("Failed to parse program: {}", e))?;

    let program = parse_result.next().ok_or("No program found in parsed result")?;

    let mut syntax = Syntax::new(program);
    syntax.analyze()?;
    syntax.optimize()?;

    let mut assembler = Assembler::new(syntax);
    assembler.assemble()?;

    Ok(())
}
