// WIP

axp::morphing_lexer! {
  @dollar: $;

  @initial_mode: Init;

  @morphs: {
    Init { StartStr => Str, StartGdStr(_) => GdStr }
    Str { EndStr => Init }
    GdStr { EndGdStr => Init }
  }

  @apply_to_all_lexer_mode_enums: {
    #[allow(clippy::enum_variant_names, unused)]
    #[logos(subpattern white_space=" \n\r\t")]
    #[logos(subpattern bad_cats=r"\p{Cc}\p{Cn}\p{Co}\pZ")]
    #[logos(subpattern bad_char="[[(?&bad_cats)]--[(?&white_space)]]")]
    #[logos(subpattern guard="[0-9a-fA-F]{0,9}")]
    #[logos(subpattern double_quote="\"")]
    #[logos(subpattern back_slash=r"\\")]
    #[logos(subpattern hash="#")]
  }

  #[logos(subpattern bad_bare=r"\(\)(?&double_quote)(?&back_slash)(?&hash):")]
  #[logos(subpattern bare="[^(?&bad_bare)(?&bad_cats)]+")]
  #[logos(subpattern comment="[[ \t][^(?&bad_cats)]]")]
  pub lexer_mode_enum Init<'source> {
    #[regex("[(?&white_space)]+", priority=3)]
    WhiteSpace,

    #[regex("#+[ \t](?&comment)+", with_slice)]
    Comment(&'source[u8]),

    #[regex("(?&bare)+", with_slice, priority=3)]
    Bare(&'source[u8]),

    #[token(":")]
    Colon,

    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[regex("(?&double_quote)#(?&guard)", with_slice)]
    StartGdStr(&'source[u8]),

    #[regex("(?&double_quote)")]
    StartStr,

    #[regex("[(?&back_slash)(?&hash)]", with_slice, priority=2)]
    #[regex("(?&bad_char)", with_slice, priority=2)]
    BadChar(&'source[u8]),

    #[regex(b".", |lexer| lexer.slice(), priority=1)]
    BadByte(&'source[u8])
  }

  #[logos(subpattern part="[^(?&bad_cats)(?&back_slash)(?&double_quote)]+")]
  pub lexer_mode_enum Str<'source> {
    #[regex("(?&part)+", with_slice)]
    Part(&'source[u8]),

    #[regex(b"(?&back_slash)[ \"enrt0]", with_slice)]
    Esc(&'source[u8]),

    #[regex(b"(?&double_quote)")]
    EndStr
  }

  pub lexer_mode_enum GdStr<'source> { // todo
    #[regex("[^(?&bad_cats)]", with_slice, priority=0)]
    Part(&'source[u8]),

    #[regex(b"(?&back_slash)")]
    StartEsc,

    #[regex(b"(?&double_quote)")]
    EndGdStr
  }
}

use logos::{Lexer, Logos, Source};
use mode::Init::*;
use mode::Str::*;

fn with_slice<'source, T: Logos<'source>>(
  lexer: &mut Lexer<'source, T>,
) -> &'source <T::Source as Source>::Slice {
  lexer.slice()
}

#[test]
fn lex_string() {
  env_logger::init();
  log::trace!("starting");

  let text = r#"a "text\nline""#.as_bytes();
  let lex = MorphingLexer::new(text);
  let tokens = lex.collect::<Vec<_>>();

  use MorphingToken::{Init, Str};
  assert_eq!(&tokens, &[
    Ok(Init(Bare(b"a"))),
    Ok(Init(WhiteSpace)),
    Ok(Init(StartStr)),
    Ok(Str(Part(b"text"))),
    Ok(Str(Esc(b"\\n"))),
    Ok(Str(Part(b"line"))),
    Ok(Str(EndStr)),
  ]);
}

#[test]
fn lex_escape() {
  let mut lex_init = mode::Init::lexer(r#""\""#.as_bytes());
  let token = lex_init.next();
  assert_eq!(token, Some(Ok(StartStr)));

  let mut lex_str = lex_init.morph::<mode::Str>();
  let token = lex_str.next();
  assert_eq!(token, Some(Ok(Esc(b"\\\""))));
}
