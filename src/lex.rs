use logos::Logos;
use std::fmt;

use crate::pretty::PrettyUtf8;

// Unicode character classes like `\pCc\pCn\pCo\pZ` combined with limited
// repetition cause the state machine to  b grow extremely large. Solution:
// accept non-utf8 data everywhere and pass them through.

// I want to create a streaming lexer, so the token should be not too large.
// Also the lexer generator seems to dislike bounded repetitions, so I limit
// them to a length of 20 bytes.

#[derive(Logos, Clone, Copy, Debug, Eq, PartialEq)]
#[logos(subpattern guard=br#"(#[^ \n\r\t\\"]{0,8})?"#)]
pub enum Base<'b> {
  #[regex(b"[ \n\r\t]{1,20}")]
  WhiteSpace,

  #[regex(b"#+[ \t][\n\r]{1,20}", slice)]
  Comment(&'b [u8]),

  #[regex(br#"[^:#\(\) \n\r\t\\"]{1,20}"#, slice)]
  Bare(&'b [u8]),

  #[token(b":")]
  Colon,

  #[token(b"(")]
  Open,

  #[token(b")")]
  Close,

  #[regex(br#"[#\\]"#, slice)]
  Special(&'b [u8]),

  #[regex(b"(?&guard)\"", slice)]
  Quoted(&'b [u8]),
}

#[derive(Logos, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Comment<'b> {
  #[regex(b"[\n\r]")]
  End,

  #[regex(b"[^\n\r]{1,20}", slice)]
  Part(&'b [u8]),
}

#[derive(Logos, Clone, Copy, Debug, Eq, PartialEq)]
#[logos(subpattern guard=br#"(#[^ \n\r\t\\"]{0,8})?"#)]
pub enum Quoted<'b> {
  #[regex(br#"[^\\"]{1,20}"#, slice)]
  Part(&'b [u8]),

  #[regex(br#"(?&guard)\\["enrt0]"#, slice)]
  #[regex(br#"(?&guard)\\x[0-9a-fA-F]{2}"#, slice)]
  #[regex(br#"(?&guard)\\u\x7b[0-9a-fA-F]{2,8}\x7d"#, slice)]
  #[regex(br#"(?&guard)\\[ \n\r\t]{1,20}\\"#, slice)]
  Esc(&'b [u8]),

  #[regex(br#"(?&guard)\\[^"enrt0xu]"#, slice)]
  BadEsc(&'b [u8]),

  #[regex(br#"(?&guard)""#, slice)]
  End(&'b [u8]),
}

fn slice<'b, L: Logos<'b>>(lex: &mut logos::Lexer<'b, L>) -> &'b [u8]
where
  <<L as Logos<'b>>::Source as logos::Source>::Slice: AsRef<[u8]>,
{
  lex.slice().as_ref()
}

#[derive(Clone, Debug)]
enum Lex<'b> {
  Base(logos::Lexer<'b, Base<'b>>),
  Comment(logos::Lexer<'b, Comment<'b>>),
  Quoted(logos::Lexer<'b, Quoted<'b>>),
}

#[derive(Clone, Debug)]
struct AxpLexer<'b> {
  lex: Lex<'b>,
  guard: &'b [u8],
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Token<'b> {
  WhiteSpace,
  Bare(&'b [u8]),
  Colon,
  Open,
  Close,
  Special(&'b [u8]),
  Quoted(&'b [u8]),
  Esc(&'b [u8]),
  BadEsc(&'b [u8]),
}

impl<'b> fmt::Debug for Token<'b> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let width = f.precision().unwrap_or(0);
    match self {
      Token::WhiteSpace => f.write_str("WhiteSpace"),
      Token::Bare(s) => write!(f, "Bare({})", s.pretty_short(width)),
      Token::Colon => f.write_str("Colon"),
      Token::Open => f.write_str("Open"),
      Token::Close => f.write_str("Close"),
      Token::Special(s) => write!(f, "Special({})", s.pretty_short(width)),
      Token::Quoted(s) => write!(f, "Quoted({})", s.pretty_short(width)),
      Token::Esc(s) => write!(f, "Esc({})", s.pretty_short(width)),
      Token::BadEsc(s) => write!(f, "BadEsc({})", s.pretty_short(width)),
    }
  }
}

impl<'b> Iterator for AxpLexer<'b> {
  type Item = Token<'b>;

  fn next(&mut self) -> Option<Self::Item> {
    fn slice_without_last(bytes: &[u8]) -> &[u8] {
      let n = if bytes.is_empty() { 0 } else { bytes.len() - 1 };
      &bytes[..n]
    }

    loop {
      match &mut self.lex {
        Lex::Base(lex_base) => {
          let token = lex_base.next();
          log::trace!("lex_base.next(): {token:?}");

          if let Some(Ok(base)) = token {
            match base {
              Base::WhiteSpace => return Some(Token::WhiteSpace),
              Base::Comment(_) => {
                self.lex = Lex::Comment(lex_base.to_owned().morph());
                continue;
              }
              Base::Bare(bare) => return Some(Token::Bare(bare)),
              Base::Colon => return Some(Token::Colon),
              Base::Open => return Some(Token::Open),
              Base::Close => return Some(Token::Close),
              Base::Special(s) => return Some(Token::Special(s)),
              Base::Quoted(guard) => {
                self.guard = slice_without_last(guard);
                self.lex = Lex::Quoted(lex_base.to_owned().morph());
                continue;
              }
            }
          } else if token.is_none() {
            return None;
          }

          unreachable!("unexpected result from lex_base.next(): {token:?}");
        }

        Lex::Comment(lex_comment) => {
          let token = lex_comment.next();
          log::trace!("lex_comment.next(): {token:?}");

          if let Some(Ok(comment)) = token {
            match comment {
              Comment::Part(_) => continue,
              Comment::End => {
                self.lex = Lex::Base(lex_comment.to_owned().morph());
                continue;
              }
            }
          } else if token.is_none() {
            return None;
          }

          unreachable!("unexpected result from lex_comment.next(): {token:?}");
        }

        Lex::Quoted(lex_quoted) => {
          let token = lex_quoted.next();
          log::trace!("lex_quoted.next(): {token:?}");

          if let Some(Ok(quoted)) = token {
            match quoted {
              Quoted::Part(s) => return Some(Token::Quoted(s)),
              Quoted::End(guard) => {
                if self.guard == slice_without_last(guard) {
                  self.lex = Lex::Base(lex_quoted.to_owned().morph());
                  continue;
                } else {
                  return Some(Token::Quoted(guard));
                }
              } // todo handle guard
              Quoted::Esc(s) => return Some(Token::Esc(s)),
              Quoted::BadEsc(s) => return Some(Token::BadEsc(s)),
            }
          } else if token.is_none() {
            return None;
          }

          unreachable!("unexpected result from lex_quoted.next(): {token:?}");
        }
      }
    }
  }
}

pub fn lex(input: &[u8]) -> impl Iterator<Item = Token> {
  AxpLexer { lex: Lex::Base(Base::lexer(input)), guard: b"" }
}

#[cfg(test)]
mod tests {
  use super::lex;
  use super::Token::{self, *};

  fn lex_bytes(input: &[u8]) -> Vec<Token<'_>> {
    let _ = env_logger::try_init();
    log::trace!("starting lex");

    lex(input).collect()
  }

  fn lex_str(input: &str) -> Vec<Token<'_>> {
    let _ = env_logger::try_init();
    log::trace!("starting lex");

    lex(input.as_bytes()).collect()
  }

  #[test]
  fn lex_bares() {
    assert_eq!(lex_str("aä.[💩"), &[Bare("aä.[💩".as_bytes())]);
    assert_eq!(
      lex_str("a\0b c(d)e:f\ng\th\ri\\j#k\""),
      &[
        Bare(b"a\0b"),
        WhiteSpace,
        Bare(b"c"),
        Open,
        Bare(b"d"),
        Close,
        Bare(b"e"),
        Colon,
        Bare(b"f"),
        WhiteSpace,
        Bare(b"g"),
        WhiteSpace,
        Bare(b"h"),
        WhiteSpace,
        Bare(b"i"),
        Special(b"\\"),
        Bare(b"j"),
        Special(b"#"),
        Bare(b"k"),
        Quoted(b"\""),
      ]
    );

    // Bares get broken up after 32 bytes
    assert_eq!(
      lex_str("0123456789abcdefghijklmnopqrstuvwxyz"),
      &[Bare(b"0123456789abcdefghijklmnopqrstuv"), Bare(b"wxyz"),]
    );
  }

  #[test]
  fn lex_whitespace() {
    // Whitespace get broken up after 20 bytes
    let mut ws = [b" \n\r\t".repeat(5), vec![b' ']].concat();
    assert_eq!(lex_bytes(&ws), &[WhiteSpace, WhiteSpace]);
    ws.pop();
    assert_eq!(lex_bytes(&ws), &[WhiteSpace]);
  }

  #[test]
  fn mixed() {
    // todo
    assert_eq!(
      lex_str(r#"a-bare "text\nline""#),
      &[
        Bare(b"a-bare"),
        WhiteSpace,
        Quoted(b"\""),
        Quoted(b"text"),
        Esc(b"\\n"),
        Quoted(b"line"),
      ]
    );
  }

  #[test]
  fn lex_escapes() {
    // todo
    assert_eq!(lex_str(r#""\"\e"#), &[Esc(b"\\\""), Esc(b"\\e")]);
  }

  #[test]
  fn lex_comments_and_witespace() {
    assert_eq!(lex_str("# co\tmment\n"), &[WhiteSpace]);

    assert_eq!(lex_bytes(b"# comment\0text"), &[]);

    // test break up of comments
  }

  // todo string guards
}
