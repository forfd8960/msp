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
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then: Block,
        else_body: Option<Block>,
    },
    For {
        init: Box<Stmt>,
        cond: Box<Expr>,
        update: Box<Stmt>,
        body: Block,
    },
    While {
        cond: Box<Expr>,
        body: Block,
    },
    Func {
        name: String,
        params: Vec<String>,
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
pub enum Expr {
    Ident(String),
    Literal(Literal),
    /// [1, 2, "c"]
    ArrLiteral(Vec<Expr>),

    FuncCall {
        fn_name: String,
        params: Vec<Expr>,
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
