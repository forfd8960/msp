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

    /// =
    Assign,

    /// (
    LeftParent,
    /// )
    RightParent,

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
