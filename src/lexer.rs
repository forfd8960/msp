use crate::{MSPResult, Token, errors::LexerErrors};

#[derive(Debug)]
pub struct Lexer {
    pub chars: Vec<char>,
    pub start: usize,
    pub current: usize,
}

impl Lexer {
    pub fn new(input: Vec<char>) -> Self {
        Self {
            chars: input,
            start: 0,
            current: 0,
        }
    }

    pub fn lex(&mut self) -> MSPResult<Vec<Token>> {
        let mut tokens = Vec::with_capacity(self.chars.len());
        loop {
            if self.is_end() {
                break;
            }

            self.start = self.current;
            let token = self.next_token()?;
            match token {
                Token::Eof => break,
                Token::WhiteSpace => {}
                _ => tokens.push(token),
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> MSPResult<Token> {
        let cur_ch = self.advance().ok_or_else(|| LexerErrors::NoTokens)?;
        println!("current: {}", cur_ch);

        match cur_ch {
            '\t' | '\n' | ' ' => Ok(Token::WhiteSpace),
            ',' => Ok(Token::Comma),
            ';' => Ok(Token::Semicolon),
            '+' => {
                if self.is_match('+') {
                    self.advance();
                    return Ok(Token::Increment);
                } else {
                    Ok(Token::Plus)
                }
            }
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Mul),
            '/' => Ok(Token::Div),
            '%' => Ok(Token::Mod),
            '<' => {
                if self.is_match('=') {
                    self.advance();
                    return Ok(Token::LtEq);
                } else {
                    Ok(Token::LessThan)
                }
            }
            '>' => {
                if self.is_match('=') {
                    self.advance();
                    return Ok(Token::GtEq);
                }
                Ok(Token::GreatThan)
            }
            '=' => {
                if self.is_match('=') {
                    self.advance();
                    return Ok(Token::Eq);
                }
                Ok(Token::Assign)
            }
            _ => Err(LexerErrors::UnknowChar(cur_ch).into()),
        }
    }

    fn is_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn is_match(&self, ch1: char) -> bool {
        self.peek().is_some_and(|ch| ch == ch1)
    }

    fn peek(&self) -> Option<char> {
        if self.is_end() {
            None
        } else {
            Some(self.chars[self.current])
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_end() {
            None
        } else {
            self.current += 1;
            Some(self.chars[self.current - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    fn fib(n: u32) -> u32 {
        if n == 1 || n == 2 {
            return 1;
        }

        fib(n - 2) + fib(n - 1)
    }
    #[test]
    fn test_fib() {
        let res = fib(5);
        assert_eq!(res, 5);
    }
}
