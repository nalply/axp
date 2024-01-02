#![allow(unused)]
use core::fmt;
use std::error::Error;

use crate::{Atom, List, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError(String);

impl Error for ParseError {}

impl ParseError {
  pub fn new<I: Into<String>>(msg: I) -> Self { ParseError(msg.into()) }
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("ParseError: ")?;
    f.write_str(&self.0)
  }
}

macro_rules! err {
  ( $( $tt:tt )+ ) => {
    ParseError::new(format!( $( $tt )+ ))
  }
}

macro_rules! throw {
  ( $( $tt:tt )+ ) => {
    Err(ParseError::new(format!( $( $tt )+ )))?
  }
}

type Parse<T> = Result<T, ParseError>;
// type Token<'s> = ParseResult<MorphingToken<'s>>;

pub fn parse(input: &[u8]) -> Parse<Value> { todo!() }

// fn parse_list<'s>(lexer: &mut impl Iterator<Item = Token<'s>>) -> Parse<List>
// {   let mut list = List(vec![]);
//   if let Some(Ok(Init(token))) = lexer.next() {
//     match token {
//       _ => throw!("unexpected token"),
//     }

//     Ok(list)
//   } else {
//     throw!("internal: unexpected lexer mode {lexer:?}")
//   }
// }

#[cfg(test)]
mod tests {
  use super::parse;

  #[test]
  fn test_parse() {
    let result = parse(b"whatever");
    println!("{result:?}");
  }
}
