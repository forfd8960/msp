#[derive(Debug, Clone, PartialEq)]
pub enum KW {
    /// fn
    Function,
    /// null
    Null,
    /// if
    If,
    /// for
    For,
    /// else
    Else,
    /// while
    While,
    /// break
    Break,
    /// continue
    Continue,
    /// return
    Return,
    /// print
    Print,
    /// strlen: return the length of a string
    StrLen,
    /// arrlen: return the length of a array
    ArrLen,
    /// append: append element to an array.
    Append,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    KeyWord(KW),
    Ident(String),
    Str(String),
    Int(i64),
    Float(f64),
    Boolean(bool),

    Plus,
    Minus,
    Mul,
    Div,
    Mod,

    /// ++
    Increment,

    /// <
    LessThan,
    /// <=
    LtEq,
    /// >
    GreatThan,
    /// >=
    GtEq,
    /// ==
    Eq,
    /// !=
    NotEq,
    /// &&
    And,
    /// ||
    Or,
    /// !
    Bang,

    /// =
    Assign,

    /// ( left parenthesis
    LeftParen,
    /// ) Right parenthesis
    RightParen,

    /// [
    LeftBracket,
    /// ]
    RightBracket,

    /// {
    LeftBrace,
    /// }
    RightBrace,

    /// ,
    Comma,
    /// ;
    Semicolon,
    /// \t \n ' '
    WhiteSpace,
    /// eof
    Eof,
}

pub fn build_kw(ident: &str) -> Option<KW> {
    match ident {
        "fn" => Some(KW::Function),
        "if" => Some(KW::If),
        "else" => Some(KW::Else),
        "for" => Some(KW::For),
        "while" => Some(KW::While),
        "return" => Some(KW::Return),
        "continue" => Some(KW::Continue),
        "break" => Some(KW::Break),
        "print" => Some(KW::Print),
        "strlen" => Some(KW::StrLen),
        "arrlen" => Some(KW::ArrLen),
        "append" => Some(KW::Append),
        "null" => Some(KW::Null),
        _ => None,
    }
}
