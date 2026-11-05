use crate::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub stmt: Stmt,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Box<Expr>),
    Block(Block),
    If {
        cond: Box<Expr>,
        then: Block,
        else_body: Option<Block>,
    },
    For {
        init: Box<Expr>,
        cond: Box<Expr>,
        update: Box<Expr>,
        body: Block,
    },
    While {
        cond: Box<Expr>,
        body: Block,
    },
    Func {
        name: Token,
        params: Vec<Token>,
        body: Block,
    },
    Return {
        value: Option<Expr>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Eq,
    NotEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Bang,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltIn {
    /// print
    Print,
    /// strlen: return the length of a string
    StrLen,
    /// arrlen: return the length of a array
    ArrLen,
    /// append: append element to an array.
    Append,
    /// null
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    BuiltIn(BuiltIn),
    Literal(Literal),
    /// [1, 2, "c"]
    ArrLiteral(Vec<Expr>),

    FuncCall {
        fn_name: Box<Expr>,
        params: Vec<Expr>,
    },

    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    Increment {
        target: Box<Expr>,
    },

    BinaryExpr {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    /// arr[i], arr[x+1]
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str(String),
    Number(i64),
    Boolean(bool),
    Null,
}
