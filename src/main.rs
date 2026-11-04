use msp::Token;

fn main() -> anyhow::Result<()> {
    println!("My Simple Programming Language");

    let gt = Token::GreatThan;
    println!(">: {:?}", gt);

    let gteq = Token::GtEq;
    println!(">=: {:?}", gteq);
    Ok(())
}
