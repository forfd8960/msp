pub mod errors;
pub mod lexer;
pub mod token;

pub use errors::MSPErrors;
pub use lexer::Lexer;
pub use token::KW;
pub use token::Token;
pub use token::build_kw;

pub type MSPResult<T> = Result<T, MSPErrors>;
