use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexerErrors {
    #[error("{0}")]
    UnknowToken(String),

    #[error("{0}")]
    UnterminatedString(String),
}

#[derive(Debug, Error)]
pub enum ParserErrors {
    #[error("{0}")]
    UnknowToken(String),

    #[error("{0}")]
    UnterminatedString(String),
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
    #[error("lexer error")]
    LexerErrors,
    #[error("parser error")]
    ParserErrors,
    #[error("eval error")]
    EvalErrors,
}
