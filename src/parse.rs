#![allow(unused)]
use core::fmt;
use std::arch::x86_64::_XCR_XFEATURE_ENABLED_MASK;
use std::error::Error;
use std::f32::consts::LOG10_2;

use crate::lex::AxpLexer;
use crate::pretty::PrettyUtf8;
use crate::{lex, Atom, Item, List, Map, Token};
use crate::{map, Token::*};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError(String);

impl Error for ParseError {}

impl ParseError {
  pub fn new<I: Into<String>>(msg: I) -> Self {
    ParseError(msg.into())
  }
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("ParseError: ")?;
    f.write_str(&self.0)
  }
}

macro_rules! err {
  ( $( $tt:tt )+ ) => {{
    fn trim(s: &str) -> &str {
      if let Some(pos) = s.rfind("::") { &s[pos + 2..] } else { s }
    }

    ParseError::new(format!("{} [{}:{}]",
      format!( $( $tt )+ ), trim(module_path!()), line!(),
    ))
  }}
}

macro_rules! throw {
  ( $( $tt:tt )+ ) => {
    Err(err!( $( $tt )+ ))?
  }
}

type Parse<T> = Result<T, ParseError>;

struct Parser<'b> {
  lexer: AxpLexer<'b>,
  token: Option<Token<'b>>,
  mode: Mode,
}

// todo handle col, line
impl<'b> Parser<'b> {
  fn next(&mut self) -> Option<Token<'b>> {
    let old_token = self.token;
    self.token = self.lexer.next();
    old_token
  }

  fn next_fluent(&mut self) -> &mut Self {
    self.next();
    self
  }

  fn skip_ws(&mut self) -> Option<Token<'b>> {
    while let Some(WhiteSpace(_) | Comment(_)) = self.token {
      self.token = self.lexer.next()
    }
    self.token
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
  Top,
  Normal,
}

pub fn parse(input: &[u8]) -> Parse<Item> {
  let mut lexer = lex(input);
  let token = lexer.next();
  parse_compound(&mut Parser { lexer, token, mode: Mode::Top })
}

fn parse_compound(parser: &mut Parser<'_>) -> Parse<Item> {
  use Item::*;

  macro_rules! push_item_and_continue {
    ( $parser:expr, $list:expr, $item:expr ) => {{
      $list.push($item.clone());
      // $parser.next();
      continue;
    }};
  }

  fn do_nothing() {}

  let mut item = Item::new_list([]);
  let top = parser.mode == Mode::Top;
  parser.mode = Mode::Normal;

  log::trace!("parse_compound");

  loop {
    let token = parser.skip_ws();
    log::trace!("parse_compound key: token={token:?} item={item:?}");

    // get key, get element or close compound
    let key = match token {
      Some(Bare(s)) => parse_bare(parser)?,
      Some(Open) => parse_compound(parser.next_fluent())?,
      Some(Quoted(s)) => parse_quoted(parser)?,
      x @ Some(Esc(_) | WhiteSpace(_) | Comment(_)) => unreachable!("{x:?}"),

      Some(Bad(s)) => throw!("bad: {}", s.pretty()),
      Some(Colon) => throw!("unexpected :"),

      Some(Close) if top => throw!("unexpected )"),
      Some(Close) => return Ok(item),

      None if top => return Ok(item),
      None => throw!("unexpected end"),
    };

    parser.next();
    let token = parser.skip_ws();
    log::trace!("parse_compound colon: token={token:?}");

    // for lists push and continue or for maps handle colon
    match (token, &mut item) {
      // on first iteration item is an empty list, mutate to map
      (Some(Colon), List(list)) if list.is_empty() => item = Item::new_map([]),

      // token will be handled in the next loop iteration
      (_, List(ref mut list)) => push_item_and_continue!(parser, list, key),

      // the colon is good for maps
      (Some(Colon), Map(_)) => do_nothing(),

      (Some(token), _) => throw!("unexpected {token}"),
      (None, _) => throw!("unexpected end"),
    }

    parser.next();
    let token = parser.skip_ws();
    log::trace!("parse_compound value: token={token:?}");

    //  maps only: get value
    let value = match token {
      Some(Bare(s)) => parse_bare(parser)?,
      Some(Open) => parse_compound(parser)?,
      Some(Quoted(s)) => parse_quoted(parser)?,

      x @ Some(Esc(_) | WhiteSpace(_) | Comment(_)) => unreachable!("{x:?}"),

      Some(Bad(s)) => throw!("bad: {}", s.pretty()),
      Some(Colon) => throw!("unexpected :"),
      Some(Close) => throw!("unexpected )"),
      None => throw!("unexpected end"),
    };

    // push entry
    match &mut item {
      Item::Map(ref mut map) => {
        map.push(key, value);
      }
      _ => unreachable!("not a map"),
    }
    parser.next();
  }

  unreachable!("loop ended without returning");
}

// todo concatenate bares
fn parse_bare(parser: &mut Parser<'_>) -> Parse<Item> {
  if let Some(Bare(s)) = parser.token {
    Ok(Item::new_atom(s))
  } else {
    throw!("not a bare")
  }
}

fn parse_quoted(parser: &mut Parser<'_>) -> Parse<Item> {
  throw!("todo")
}

#[cfg(test)]
mod tests {
  use super::parse;
  use crate::Item;

  #[test]
  fn run() {
    // axlog::init("t");
    let result = parse(b"whatever 2nd () (a)");
    log::trace!("{result:?}");
  }

  #[test]
  fn test_parse() {
    assert_eq!(parse(b""), Ok(Item::nil()));
    assert_eq!(parse(b"a"), Ok(Item::new_list([Item::new_atom(b"a")])));
    assert_eq!(
      parse(b" x y "),
      Ok(Item::new_list([Item::new_atom(b"x"), Item::new_atom(b"y")]))
    );
    assert_eq!(
      parse(b" x () "),
      Ok(Item::new_list([Item::new_atom(b"x"), Item::new_list([])]))
    );
    assert_eq!(
      parse(" Schönen ( Tag ) ! ".as_bytes()),
      Ok(Item::new_list([
        Item::new_atom("Schönen".as_bytes()),
        Item::new_list([Item::new_atom(b"Tag")]),
        Item::new_atom(b"!"),
      ]))
    );
  }
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
