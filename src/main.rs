use anyhow::anyhow;
use msp::Lexer;

fn main() -> anyhow::Result<()> {
    println!("My Simple Programming Language");

    let mut rl = rustyline::DefaultEditor::new()?;

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                println!("Line: {:?}", line);
                let input = line.chars().collect::<Vec<char>>();
                let mut lexer = Lexer::new(input);
                match lexer.lex() {
                    Ok(tokens) => println!("tokens: {:?}", tokens),
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
