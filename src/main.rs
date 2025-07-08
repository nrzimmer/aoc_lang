mod assembler;
mod lexer;
mod syntax;

use crate::assembler::Assembler;
use crate::lexer::{Lexer, Rule};
use crate::syntax::Syntax;
use pest::Parser;
use std::env::temp_dir;
use std::process::Command;
use std::{env, fs};

#[cfg(feature = "debug")]
const FILE_INPUT: Option<&str> = Some("assets/2015_01.aoc");

#[cfg(not(feature = "debug"))]
const FILE_INPUT: Option<&str> = None;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = if let Some(file_path) = FILE_INPUT {
        file_path.to_string()
    } else {
        let args: Vec<String> = env::args().collect();
        let file_path = args.get(1).ok_or("Usage: program <file_path>")?;
        file_path.to_string()
    };

    let file_content = fs::read_to_string(file_path)?;

    let mut parse_result = Lexer::parse(Rule::program, &file_content).map_err(|e| format!("Failed to parse program: {}", e))?;

    let program = parse_result.next().ok_or("No program found in parsed result")?;

    let mut syntax = Syntax::new(program);
    syntax.analyze()?;
    syntax.optimize()?;

    let mut assembler = Assembler::new(syntax);
    let code = assembler.assemble()?;

    println!("{}", code);

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let mut asm_file = temp_dir();
    asm_file.push(format!("output_{}.s", time));
    fs::write(&asm_file, code)?;

    let mut obj_file = temp_dir();
    obj_file.push(format!("output_{}.o", time));
    let output = Command::new("gcc")
        .arg(asm_file)
        .arg("./assets/utils.c")
        .arg("-o")
        .arg("./a.out")
        .output()
        .map_err(|e| format!("Failed to execute gcc: {}", e))?;

    if !output.status.success() {
        return Err(format!("GCC compilation failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }

    Ok(())
}
