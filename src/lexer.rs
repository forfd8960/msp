use crate::{MSPResult, Token, build_kw, errors::LexerErrors};

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
            '(' => Ok(Token::LeftParen),
            ')' => Ok(Token::RightParen),
            '[' => Ok(Token::LeftBracket),
            ']' => Ok(Token::RightBracket),
            '{' => Ok(Token::LeftBrace),
            '}' => Ok(Token::RightBrace),
            '&' => {
                if self.is_match('&') {
                    self.advance();
                    return Ok(Token::And);
                } else {
                    Err(LexerErrors::UnknowChar('&').into())
                }
            }
            '|' => {
                if self.is_match('|') {
                    self.advance();
                    return Ok(Token::Or);
                } else {
                    Err(LexerErrors::UnknowChar('|').into())
                }
            }
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
            '!' => {
                if self.is_match('=') {
                    self.advance();
                    return Ok(Token::NotEq);
                }

                Ok(Token::Bang)
            }
            '"' => self.parse_string(),
            _ => {
                if cur_ch.is_ascii_digit() {
                    return self.parse_number();
                } else if cur_ch.is_alphabetic() {
                    return self.parse_ident();
                }
                Err(LexerErrors::UnknowChar(cur_ch).into())
            }
        }
    }

    fn parse_string(&mut self) -> MSPResult<Token> {
        let mut end = false;
        while let Some(c) = self.peek() {
            if c == '"' {
                end = true;
                break;
            }
            self.advance();
        }

        if !end && self.is_end() {
            return Err(LexerErrors::UnterminatedString.into());
        }

        self.advance();
        let s = String::from_iter(&self.chars[self.start + 1..self.current - 1]);

        Ok(Token::Str(s))
    }

    fn parse_number(&mut self) -> MSPResult<Token> {
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            self.advance();
        }

        match String::from_iter(&self.chars[self.start..self.current]).parse::<i64>() {
            Ok(n) => Ok(Token::Int(n)),
            Err(e) => Err(LexerErrors::InvalidNumber(e).into()),
        }
    }

    fn parse_ident(&mut self) -> MSPResult<Token> {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c.eq(&'_') {
                self.advance();
                continue;
            }

            break;
        }

        let ident = String::from_iter(&self.chars[self.start..self.current]);
        if let Some(kw) = build_kw(&ident) {
            return Ok(Token::KeyWord(kw));
        }
        match ident.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            _ => Ok(Token::Ident(ident)),
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
