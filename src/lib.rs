pub mod errors;
pub mod lexer;
pub mod token;

pub use errors::MSPErrors;
pub use token::KW;
pub use token::Token;

pub type MSPResult<T> = Result<T, MSPErrors>;
