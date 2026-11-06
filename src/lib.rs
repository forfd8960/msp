pub mod ast;
pub mod errors;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod token;

pub use ast::Program;
pub use errors::MSPErrors;
pub use lexer::Lexer;
pub use token::KW;
pub use token::Token;
pub use token::build_kw;

pub type MSPResult<T> = Result<T, MSPErrors>;
