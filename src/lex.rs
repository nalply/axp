use crate::morphing_lexer;

// I tried to disallow bad characters like
// \pCc\pCn\pCo\pZ but combined with limited repetitions the state machine
// got extremely large. Now the only disallowed character is NUL, for testing
// After lexing the parser needs to check for bad characters.

morphing_lexer! {
  @dollar: $;

  @initial_mode: Init;

  @morphs: {
    Init { StartComment(_) => Comment, StartStr(_) => Str }
    Comment { EndComment => Init }
    Str { EndStr => Init }
  }

  @apply_to_all_lexer_mode_enums: {
    #[allow(clippy::enum_variant_names, unused)]
    #[logos(subpattern comment_char=b"[^\x00\n\r]")]
    #[logos(subpattern double_quote=b"\"")]
    #[logos(subpattern back_slash=br"\\")]
    #[logos(subpattern opt_guard=b"(#[_#0-9a-fA-F]{0,9})?")]
  }

  #[logos(subpattern not_bare=br"\x00:#\(\)(?&double_quote)(?&back_slash)")]
  #[logos(subpattern bare=b"[^(?&not_bare) \n\r\t]")]
  pub lexer_mode_enum Init<'source> {
    #[regex(b"[ \n\r\t]{1,99}")]
    WhiteSpace,

    #[regex(b"#+[ \t](?&comment_char){1,99}", with_slice)]
    StartComment(&'source[u8]),

    #[regex(b"(?&bare){1,32}", with_slice)]
    BarePart(&'source[u8]),

    #[token(b":")]
    Colon,

    #[token(b"(")]
    ParenOpen,

    #[token(b")")]
    ParenClose,

    #[regex(b"(?&opt_guard)(?&double_quote)", with_slice)]
    StartStr(&'source[u8]),

    #[regex(b"[\x00#\\\\]", with_slice)]
    Bad(&'source[u8]),
  }

  pub lexer_mode_enum Comment<'source> {
    #[regex(b"[\r\n]")]
    EndComment,

    #[regex(b"(?&comment_char){1,99}", with_slice)]
    CommentPart(&'source[u8]),

    #[regex(b"\x00", with_slice)]
    BadComment(&'source[u8]),
  }

  #[logos(subpattern part=b"[^(?&back_slash)(?&double_quote)]+")]
  #[logos(subpattern start_esc=b"(?&opt_guard)(?&back_slash)")]
  pub lexer_mode_enum Str<'source> {
    #[regex(b"(?&part){1,19}", with_slice)]
    StrPart(&'source[u8]),

    #[regex(b"(?&start_esc)[ \"enrt0]", with_slice)]
    #[regex(b"(?&start_esc)x[0-9a-fA-F]{2}", with_slice)]
    #[regex(b"(?&start_esc)u\\{[0-9a-fA-F]{2,8}\\}", with_slice)]
    StrEsc(&'source[u8]),

    #[regex(b"(?&opt_guard)(?&double_quote)")]
    EndStr,

    #[regex(b".", with_slice, priority=0)]
    BadStr(&'source[u8]),
  }
}

use logos::{Lexer, Logos, Source};

fn with_slice<'source, T: Logos<'source>>(
  lexer: &mut Lexer<'source, T>,
) -> &'source <T::Source as Source>::Slice {
  lexer.slice()
}

#[cfg(test)]
mod tests {
  use super::mode::Comment::*;
  use super::mode::Init::*;
  use super::mode::Str::*;
  use super::MorphingLexer;
  use super::MorphingToken::{self, Comment, Init, Str};

  fn lex_bytes(input: &[u8]) -> Vec<MorphingToken<'_>> {
    let _ = env_logger::try_init();
    log::trace!("starting lex");

    MorphingLexer::new(input).map(|token| token.unwrap()).collect()
  }

  fn lex_str(input: &str) -> Vec<MorphingToken<'_>> {
    let _ = env_logger::try_init();
    log::trace!("starting lex");

    MorphingLexer::new(input.as_bytes()).map(|token| token.unwrap()).collect()
  }

  #[test]
  fn lex_bares() {
    assert_eq!(lex_str("aä.[💩"), &[Init(BarePart("aä.[💩".as_bytes()))]);
    assert_eq!(lex_str("a\0b c(d)e:f\ng\th\ri\\j#k\""), &[
      Init(BarePart(b"a")),
      Init(Bad(b"\0")),
      Init(BarePart(b"b")),
      Init(WhiteSpace),
      Init(BarePart(b"c")),
      Init(ParenOpen),
      Init(BarePart(b"d")),
      Init(ParenClose),
      Init(BarePart(b"e")),
      Init(Colon),
      Init(BarePart(b"f")),
      Init(WhiteSpace),
      Init(BarePart(b"g")),
      Init(WhiteSpace),
      Init(BarePart(b"h")),
      Init(WhiteSpace),
      Init(BarePart(b"i")),
      Init(Bad(b"\\")),
      Init(BarePart(b"j")),
      Init(Bad(b"#")),
      Init(BarePart(b"k")),
      Init(StartStr(b"\"")),
    ]);

    // Bares get broken up after 32 bytes
    assert_eq!(lex_str("0123456789abcdefghijklmnopqrstuvwxyz"), &[
      Init(BarePart(b"0123456789abcdefghijklmnopqrstuv")),
      Init(BarePart(b"wxyz")),
    ]);
  }

  #[test]
  fn lex_whitespace() {
    // Whitespace get broken up after 99 bytes
    let mut ws = b" \n\r\t".repeat(25);
    assert_eq!(lex_bytes(&ws), &[Init(WhiteSpace), Init(WhiteSpace)]);
    ws.pop();
    assert_eq!(lex_bytes(&ws), &[Init(WhiteSpace)]);
  }

  #[test]
  fn mixed() {
    // todo
    assert_eq!(lex_str(r#"a-bare "text\nline""#), &[
      Init(BarePart(b"a-bare")),
      Init(WhiteSpace),
      Init(StartStr(b"\"")),
      Str(StrPart(b"text")),
      Str(StrEsc(b"\\n")),
      Str(StrPart(b"line")),
      Str(EndStr),
    ]);
  }

  #[test]
  fn lex_escapes() {
    // todo
    assert_eq!(lex_str(r#""\"\e"#), &[
      Init(StartStr(b"\"")),
      Str(StrEsc(b"\\\"")),
      Str(StrEsc(b"\\e")),
    ]);
  }

  #[test]
  fn lex_comments() {
    assert_eq!(lex_str("# co\tmment\n"), &[
      Init(StartComment(b"# co\tmment")),
      Comment(EndComment),
    ]);

    assert_eq!(lex_bytes(b"# comment\0text"), &[
      Init(StartComment(b"# comment")),
      Comment(BadComment(b"\0")),
      Comment(CommentPart(b"text")),
    ]);

    // Comments get breaked up after 99 bytes
    fn comment(n: usize) -> String { format!("# {}\n", "-".repeat(n)) }
    let dashes = comment(99);
    let dashes = dashes.as_bytes();
    assert_eq!(lex_bytes(dashes), &[
      Init(StartComment(&dashes[..=100])),
      Comment(EndComment),
    ]);
    let dashes = comment(100);
    let dashes = dashes.as_bytes();
    assert_eq!(lex_bytes(dashes), &[
      Init(StartComment(&dashes[..=100])),
      Comment(CommentPart(b"-")),
      Comment(EndComment),
    ]);
    let dashes = comment(198);
    let dashes = dashes.as_bytes();
    assert_eq!(lex_bytes(dashes), &[
      Init(StartComment(&dashes[..=100])),
      Comment(CommentPart(&dashes[100..=198])),
      Comment(EndComment),
    ]);
    let dashes = comment(199);
    let dashes = dashes.as_bytes();
    assert_eq!(lex_bytes(dashes), &[
      Init(StartComment(&dashes[..=100])),
      Comment(CommentPart(&dashes[100..=198])),
      Comment(CommentPart(b"-")),
      Comment(EndComment),
    ]);
  }

  // todo string guards
}
