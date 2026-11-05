use std::num::ParseIntError;

use thiserror::Error;

use crate::Token;

#[derive(Debug, Error)]
pub enum LexerErrors {
    #[error("unknow char: {0}")]
    UnknowChar(char),

    #[error("no tokens")]
    NoTokens,

    #[error("unterminated string")]
    UnterminatedString,

    #[error("invalid number")]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Debug, Error)]
pub enum ParserErrors {
    #[error("expect {0}, but get: {1}")]
    ExpectToken(String, String),

    #[error("no tokens")]
    EofToken,

    #[error("invalid assign target")]
    InvalidAssignTarget,

    #[error("invalid binary expr: {0}")]
    InvalidBinary(String),

    #[error("unknown builtin")]
    UnknowBuiltIn,
    #[error("unknown primary")]
    UnknowPrimary,
}

#[derive(Debug, Error)]
pub enum EvalErrors {
    #[error("{0}")]
    UnknowToken(String),

    #[error("{0}")]
    UnterminatedString(String),
}

#[derive(Debug, Error)]
pub enum MSPErrors {
    #[error(transparent)]
    Lexer(#[from] LexerErrors),
    #[error(transparent)]
    Parser(#[from] ParserErrors),
    #[error(transparent)]
    Eval(#[from] EvalErrors),
}
