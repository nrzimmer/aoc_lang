use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "aoc.pest"]
pub struct Lexer;
