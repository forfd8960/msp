use std::{io, num::ParseIntError};

use thiserror::Error;

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
    #[error("unsupport stmt: {0}")]
    UnsupportStmt(String),

    #[error("require ident")]
    RequireIdent,

    #[error("expect stmt: {0}")]
    ExpectStmt(String),

    #[error("expect value: {0}")]
    ExpectValue(String),

    #[error("value not found: {0}")]
    VlaueNotFound(String),

    #[error("invalid syntax: {0}")]
    InvalidSyntax(String),
}

#[derive(Debug, Error)]
pub enum MSPErrors {
    #[error(transparent)]
    Lexer(#[from] LexerErrors),
    #[error(transparent)]
    Parser(#[from] ParserErrors),
    #[error(transparent)]
    Eval(#[from] EvalErrors),

    #[error("io error:{0}")]
    IOError(#[from] io::Error),
}
