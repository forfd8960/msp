use crate::{
    KW, MSPResult, Token,
    ast::{BinOp, Block, Expr, Fn, For, If, Literal, Node, Program, Stmt, UnaryOp},
    errors::ParserErrors,
};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
}

pub fn new(tokens: Vec<Token>) -> Parser {
    Parser {
        tokens,
        start: 0,
        current: 0,
    }
}

impl Parser {
    pub fn parse(&mut self) -> MSPResult<Program> {
        let mut nodes = Vec::new();
        loop {
            if self.is_end() {
                break;
            }

            match self.parse_stmt() {
                Ok(stmt) => {
                    nodes.push(Node {
                        stmt,
                        line: 0,
                        col: 0,
                    });
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Program { nodes })
    }

    fn parse_stmt(&mut self) -> MSPResult<Stmt> {
        let cur = self.peek().ok_or_else(|| ParserErrors::EofToken)?;
        match cur {
            Token::KeyWord(KW::Function) => self.parse_func(),
            Token::KeyWord(KW::If) => self.parse_if(),
            Token::KeyWord(KW::For) => self.parse_for(),
            Token::KeyWord(KW::While) => self.parse_while(),
            Token::KeyWord(KW::Return) => self.parse_return(),
            Token::LeftBrace => Ok(Stmt::Block(self.parse_block()?)),
            Token::KeyWord(KW::Break) => {
                self.advance();
                self.consume(Token::Semicolon)?;
                Ok(Stmt::Break)
            }
            Token::KeyWord(KW::Continue) => {
                self.advance();
                self.consume(Token::Semicolon)?;
                Ok(Stmt::Continue)
            }
            _ => {
                let expr = self.parse_expression()?;
                self.consume(Token::Semicolon)?;
                Ok(Stmt::Expr(Box::new(expr)))
            }
        }
    }

    fn is_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn is_match(&self, tk: Token) -> bool {
        if self.is_end() {
            return false;
        }

        if let Some(token) = self.peek() {
            if token.eq(&tk) {
                return true;
            }
        }
        false
    }

    fn consume(&mut self, t: Token) -> MSPResult<()> {
        if self.is_end() {
            return Err(ParserErrors::EofToken.into());
        }

        if let Some(token) = self.peek() {
            if token.eq(&t) {
                self.advance();
                return Ok(());
            } else {
                return Err(
                    ParserErrors::ExpectToken(format!("{:?}", t), format!("{:?}", token)).into(),
                );
            }
        } else {
            return Err(ParserErrors::EofToken.into());
        }
    }

    fn peek(&self) -> Option<Token> {
        if self.is_end() {
            None
        } else {
            Some(self.tokens[self.current].clone())
        }
    }

    fn advance(&mut self) -> Option<Token> {
        if self.is_end() {
            None
        } else {
            self.current += 1;
            Some(self.tokens[self.current - 1].clone())
        }
    }

    fn parse_func(&mut self) -> MSPResult<Stmt> {
        self.advance();

        let name = self.advance().ok_or_else(|| ParserErrors::EofToken)?;
        self.consume(Token::LeftParen)?;

        let mut params = Vec::new();
        loop {
            if self.is_match(Token::RightParen) {
                break;
            }

            params.push(self.advance().ok_or_else(|| ParserErrors::EofToken)?);
            if self.is_match(Token::RightParen) {
                break;
            }

            self.consume(Token::Comma)?;
        }

        self.consume(Token::RightParen)?;
        Ok(Stmt::Func(Fn {
            name,
            params,
            body: self.parse_block()?,
        }))
    }

    /// { stmts }
    fn parse_block(&mut self) -> MSPResult<Block> {
        let mut stmts = Vec::new();
        self.consume(Token::LeftBrace)?;
        loop {
            if self.is_match(Token::RightBrace) {
                break;
            }

            stmts.push(self.parse_stmt()?);
        }
        self.consume(Token::RightBrace)?;
        Ok(Block { stmts })
    }

    fn parse_if(&mut self) -> MSPResult<Stmt> {
        self.advance();
        let mut else_block = None;
        let cond = self.parse_expression()?;
        let then = self.parse_block()?;
        if self.is_match(Token::KeyWord(KW::Else)) {
            self.advance();
            else_block = Some(self.parse_block()?);
        }

        Ok(Stmt::If(If {
            cond: Box::new(cond),
            then,
            else_body: else_block,
        }))
    }

    fn parse_for(&mut self) -> MSPResult<Stmt> {
        self.advance();
        // for var init
        let var_init = self.parse_expression()?;
        self.consume(Token::Semicolon)?;

        let cond = self.parse_expression()?;
        self.consume(Token::Semicolon)?;

        let update = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt::For(For {
            init: Box::new(var_init),
            cond: Box::new(cond),
            update: Box::new(update),
            body,
        }))
    }

    fn parse_while(&mut self) -> MSPResult<Stmt> {
        self.advance();
        let cond = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Stmt::While {
            cond: Box::new(cond),
            body: body,
        })
    }

    fn parse_return(&mut self) -> MSPResult<Stmt> {
        self.advance();

        if self.is_match(Token::Semicolon) {
            self.advance();
            Ok(Stmt::Return { value: None })
        } else {
            let val = self.parse_expression()?;
            self.consume(Token::Semicolon)?;
            Ok(Stmt::Return { value: Some(val) })
        }
    }

    /// parse assign and other expression
    fn parse_expression(&mut self) -> MSPResult<Expr> {
        println!("parsing expression...");

        let target = self.parse_or()?;
        if !self.is_match(Token::Assign) {
            return Ok(target);
        }

        self.advance();
        if !matches!(target, Expr::Ident(_) | Expr::Index { .. }) {
            return Err(ParserErrors::InvalidAssignTarget.into());
        }

        let value = self.parse_expression()?;
        Ok(Expr::Assign {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    /// parse ||
    fn parse_or(&mut self) -> MSPResult<Expr> {
        println!("parsing or...");
        let mut left = self.parse_and()?;
        loop {
            if self.is_end() || !self.is_match(Token::Or) {
                break;
            }

            self.consume(Token::Or)?;
            let right = self.parse_and()?;
            left = Expr::BinaryExpr {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> MSPResult<Expr> {
        println!("parsing and...");

        let mut left = self.parse_equality()?;
        loop {
            if self.is_end() || !self.is_match(Token::And) {
                break;
            }

            self.consume(Token::And)?;

            let right = self.parse_equality()?;
            left = Expr::BinaryExpr {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// parse == and !=
    fn parse_equality(&mut self) -> MSPResult<Expr> {
        println!("parse equality...");

        let left = self.parse_comparison()?;

        if self.is_match(Token::Eq) || self.is_match(Token::NotEq) {
            let op = self.advance().unwrap();

            let right = self.parse_comparison()?;
            let binop = match op {
                Token::Eq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                _ => return Ok(left),
            };

            return Ok(Expr::BinaryExpr {
                op: binop,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    /// parse > >= < <=
    fn parse_comparison(&mut self) -> MSPResult<Expr> {
        let left = self.parse_term()?;
        if self.peek().is_some_and(|v| {
            matches!(
                v,
                Token::LessThan | Token::LtEq | Token::GreatThan | Token::GtEq
            )
        }) {
            let op = self.advance().unwrap();
            let right = self.parse_term()?;

            let binop = match op {
                Token::LessThan => BinOp::Lt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                Token::GreatThan => BinOp::Gt,
                _ => return Ok(left),
            };

            return Ok(Expr::BinaryExpr {
                op: binop,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    // parse + -
    fn parse_term(&mut self) -> MSPResult<Expr> {
        let mut left = self.parse_factor()?;
        loop {
            if !self
                .peek()
                .is_some_and(|v| matches!(v, Token::Plus | Token::Minus))
            {
                break;
            }

            let op = self.advance().unwrap();
            let right = self.parse_factor()?;

            let binop = match op {
                Token::Plus => BinOp::Plus,
                Token::Minus => BinOp::Minus,
                _ => return Ok(left),
            };

            left = Expr::BinaryExpr {
                op: binop,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// parse * / %
    fn parse_factor(&mut self) -> MSPResult<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            if !self
                .peek()
                .is_some_and(|v| matches!(v, Token::Mul | Token::Div | Token::Mod))
            {
                break;
            }

            let op = self.advance().unwrap();
            let right = self.parse_unary()?;

            let binop = match op {
                Token::Div => BinOp::Div,
                Token::Mul => BinOp::Mul,
                Token::Mod => BinOp::Mod,
                _ => return Ok(left),
            };

            left = Expr::BinaryExpr {
                op: binop,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// parse +xxx -xxx !xx
    fn parse_unary(&mut self) -> MSPResult<Expr> {
        if !self
            .peek()
            .is_some_and(|v| matches!(v, Token::Plus | Token::Minus | Token::Bang))
        {
            return self.parse_postfix();
        }

        let op = self.advance().unwrap();
        let right = self.parse_postfix()?;

        let unaryop = match op {
            Token::Plus => UnaryOp::Plus,
            Token::Minus => UnaryOp::Minus,
            Token::Bang => UnaryOp::Bang,
            _ => return self.parse_postfix(),
        };

        Ok(Expr::Unary {
            op: unaryop,
            expr: Box::new(right),
        })
    }

    /// parse func calls: x();
    /// index: arr[i]
    /// postfix: i++
    fn parse_postfix(&mut self) -> MSPResult<Expr> {
        let pri = self.parse_primary()?;

        if self
            .peek()
            .is_some_and(|v| matches!(v, Token::LeftParen | Token::LeftBracket | Token::Increment))
        {
            match self.advance().unwrap() {
                Token::LeftParen => self.parse_fn_call(pri),
                Token::LeftBracket => self.parse_index(pri),
                Token::Increment => Ok(Expr::Increment {
                    target: Box::new(pri),
                }),
                _ => return self.parse_primary(),
            }
        } else {
            Ok(pri)
        }
    }

    fn parse_fn_call(&mut self, expr: Expr) -> MSPResult<Expr> {
        let mut params = Vec::new();
        params.push(self.parse_expression()?);
        loop {
            if self.is_match(Token::RightParen) {
                break;
            }

            self.consume(Token::Comma)?;
            params.push(self.parse_expression()?);
        }
        self.consume(Token::RightParen)?;

        Ok(Expr::FuncCall {
            fn_name: Box::new(expr),
            params,
        })
    }

    fn parse_index(&mut self, expr: Expr) -> MSPResult<Expr> {
        let idx = self.parse_expression()?;
        self.consume(Token::RightBracket)?;
        Ok(Expr::Index {
            target: Box::new(expr),
            index: Box::new(idx),
        })
    }

    /// parse string, number, boolean literals
    /// identifiers
    /// arrays
    /// parenthesized expressions
    fn parse_primary(&mut self) -> MSPResult<Expr> {
        if let Some(tk) = self.advance() {
            let expr = match tk {
                Token::Str(s) => Expr::Literal(Literal::Str(s)),
                Token::Int(n) => Expr::Literal(Literal::Number(n)),
                Token::Boolean(b) => Expr::Literal(Literal::Boolean(b)),
                Token::Ident(s) => Expr::Ident(s),
                Token::LeftParen => {
                    let expr = self.parse_expression()?;
                    self.consume(Token::RightParen)?;
                    expr
                }
                Token::LeftBracket => self.parse_array()?,
                Token::KeyWord(KW::Null) => Expr::Literal(Literal::Null),
                Token::KeyWord(kw) => Expr::Ident(parse_builtin(kw)?),
                _ => return Err(ParserErrors::UnknowPrimary.into()),
            };

            return Ok(expr);
        }

        Err(ParserErrors::EofToken.into())
    }

    fn parse_array(&mut self) -> MSPResult<Expr> {
        let mut elements = Vec::new();
        while !self.is_match(Token::RightBracket) {
            elements.push(self.parse_expression()?);
            if self.is_match(Token::RightBracket) {
                break;
            }

            self.consume(Token::Comma)?;
        }

        self.consume(Token::RightBracket)?;
        Ok(Expr::ArrLiteral(elements))
    }
}

fn parse_builtin(kw: KW) -> MSPResult<String> {
    match kw {
        KW::Print => Ok("print".into()),
        KW::StrLen => Ok("strlen".into()),
        KW::ArrLen => Ok("arrlen".into()),
        KW::Append => Ok("append".into()),
        _ => Err(ParserErrors::UnknowBuiltIn.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Lexer, MSPErrors};

    fn number(value: i64) -> Expr {
        Expr::Literal(Literal::Number(value))
    }

    fn binary(op: BinOp, left: Expr, right: Expr) -> Expr {
        Expr::BinaryExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    #[test]
    fn parses_grouped_operands() {
        let tokens = Lexer::new("(1+2) / (9 * 6)".chars().collect())
            .lex()
            .unwrap();
        let mut parser = new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.nodes.len(), 1);
        assert_eq!(
            program.nodes[0].stmt,
            Stmt::Expr(Box::new(binary(
                BinOp::Div,
                binary(BinOp::Plus, number(1), number(2)),
                binary(BinOp::Mul, number(9), number(6)),
            )))
        );
        assert!(parser.is_end());
    }

    #[test]
    fn parses_nested_grouping_with_unary_and_precedence() {
        let tokens = Lexer::new("-(1 + (2 * 3)) + 4 * 5".chars().collect())
            .lex()
            .unwrap();
        let mut parser = new(tokens);
        let expr = parser.parse_expression().unwrap();

        assert_eq!(
            expr,
            binary(
                BinOp::Plus,
                Expr::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(binary(
                        BinOp::Plus,
                        number(1),
                        binary(BinOp::Mul, number(2), number(3)),
                    )),
                },
                binary(BinOp::Mul, number(4), number(5)),
            )
        );
        assert!(parser.is_end());
    }

    #[test]
    fn grouping_requires_a_closing_parenthesis() {
        for input in ["(1 + 2", "(1 + 2]"] {
            let tokens = Lexer::new(input.chars().collect()).lex().unwrap();
            let mut parser = new(tokens);
            assert!(matches!(
                parser.parse_expression(),
                Err(MSPErrors::Parser(ParserErrors::EofToken))
                    | Err(MSPErrors::Parser(ParserErrors::ExpectToken(_, _)))
            ));
            let tokens = Lexer::new(input.chars().collect()).lex().unwrap();
            assert!(new(tokens).parse().is_err());
        }
    }
}
