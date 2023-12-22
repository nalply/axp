#![feature(lazy_cell)]
#![forbid(unsafe_code)]

mod lex;
mod morphing_lexer;
pub mod primitive;
pub mod shorten_lossy;
mod value;

pub use value::*;
