axp::morphing_lexer! {
  @dollar: $;
  @initial_mode: Init;
  @morphs: { Init { StartStr => Str } Str { EndStr => Init } }
  @apply_to_all_lexer_mode_enums: {}
  pub lexer_mode_enum Init<'s> {
    #[regex("[ \n\r\t]", logos::skip)]
    WhiteSpace,
    #[token("\"")]
    StartStr,
    #[regex(r"\w+", |lexer| lexer.slice())]
    Ident(&'s str),
  }
  pub lexer_mode_enum Str<'s> {
    #[regex(r#"[[^\pC\pZ"][ \t]]+"#, |lexer| lexer.slice())]
    StrContents(&'s str),
    #[token("\"")]
    EndStr,
  }
}

#[test]
fn test() {
  let lex = MorphingLexer::new("hello \"world\"");
  let tokens = lex.collect::<Vec<_>>();
  println!("{tokens:?}");

  use mode::Init::*;
  use mode::Str::*;
  use MorphingToken::{Init, Str};
  assert_eq!(tokens, &[
    Ok(Init(Ident("hello"))),
    Ok(Init(StartStr)),
    Ok(Str(StrContents("world"))),
    Ok(Str(EndStr)),
  ]);
}
