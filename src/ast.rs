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
pub struct Fn {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub init: Box<Expr>,
    pub cond: Box<Expr>,
    pub update: Box<Expr>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub cond: Box<Expr>,
    pub then: Block,
    pub else_body: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Box<Expr>),
    Block(Block),
    If(If),
    For(For),
    While { cond: Box<Expr>, body: Block },

    Func(Fn),

    Return { value: Option<Expr> },
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
pub enum Expr {
    Ident(String),
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
    Char(char),
    Number(i64),
    Boolean(bool),
    Null,
}
