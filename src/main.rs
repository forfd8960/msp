use std::{fs::File, io::Read};

use anyhow::anyhow;
use msp::{
    Lexer, MSPResult,
    interpreter::{Env, Interpreter},
    parser::{self},
};

fn main() -> anyhow::Result<()> {
    println!("My Simple Programming Language");

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 2 {
        run_file(&args[1])?;
        return Ok(());
    }

    let mut rl = rustyline::DefaultEditor::new()?;
    let env = Env::new(None);
    let mut interpreter = Interpreter::new(env);

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                println!("Line: {:?}", line);
                let input = line.chars().collect::<Vec<char>>();
                let mut lexer = Lexer::new(input);
                match lexer.lex() {
                    Ok(tokens) => {
                        println!("tokens: {:?}", tokens);
                        let mut parser = parser::new(tokens);
                        match parser.parse() {
                            Ok(program) => {
                                println!("program: {:?}", program);
                                match interpreter.execute(&program) {
                                    Ok(v) => println!("value: {:?}", v),
                                    Err(e) => println!("execute err: {}", e),
                                }
                            }
                            Err(e) => println!("parse error: {}", e),
                        }
                    }
                    Err(e) => eprintln!("lexer error: {e}"),
                }
            }

            Err(e) => {
                println!("No input");
                return Err(anyhow!(e));
            }
        }
    }
}

fn run_file(path: &str) -> MSPResult<()> {
    let mut file = File::open(path)?;

    let mut code = String::new();
    file.read_to_string(&mut code)?;

    let lines = code.chars().collect::<Vec<char>>();

    let mut lexer = Lexer::new(lines);
    let env = Env::new(None);
    let mut interpreter = Interpreter::new(env);

    match lexer.lex() {
        Ok(tokens) => {
            println!("tokens: {:?}", tokens);
            let mut parser = parser::new(tokens);
            match parser.parse() {
                Ok(program) => {
                    println!("program: {:?}", program);
                    match interpreter.execute(&program) {
                        Ok(v) => println!("value: {:?}", v),
                        Err(e) => println!("execute err: {}", e),
                    }
                }
                Err(e) => println!("parse error: {}", e),
            }
        }
        Err(e) => eprintln!("lexer error: {e}"),
    }
    Ok(())
}
