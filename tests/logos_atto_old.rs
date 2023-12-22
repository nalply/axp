// cargo-deps: logos, log, env_logger

#[macro_export]
macro_rules! morphing_lexer {
  (
    @dollar: $d:tt;
    @initial_mode: $init:ident;
    @morphs: {
      $(
        $mode:ident { $( $token:pat => $target:ident $(,)? ),+ }
      )+
    }
    @apply_to_all_lexer_mode_enums: { $( #[ $( $common_meta:tt )+ ] )* }

    $(
      $( #[ $( $meta:tt )+ ] )*
      pub lexer_mode_enum $name:ident $(< $lt:lifetime >)? {
        $( $tt:tt )+
      }
    )+
  ) => {
    /// A super token over all tokens of all lexer modes: an enum of enums:
    /// the enum of lexer modes and nested the enum of the tokens of the
    /// lexer modes. Access it like this: `Mode(Token)` where `Token` might
    /// contain metadata. Example `GuardStringMode(StringPart(slice))`.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum MorphingToken<'source> {
      $( $name(mode::$name<'source>), )+
    }

    /// This lexer wraps the current lexer mode similar to `ModeBridge` in
    /// `tests/tests/lexer_modes.rs` in the Logos repository. It can switch
    /// seamlessly between modes and therefore iterate over [MorphingToken],
    /// such that `MorphingToken::new(input).collect::<Vec<_>>()` just works.
    #[derive(Clone, Debug)]
    pub struct MorphingLexer<'source> {
      lexer_mode: LexerMode<'source>,
    }

    /// The enum of lexer modes lexers. Each variant is created from the macro
    /// repetition from `pub lexer_mode_enum $name ...` where the variant's name
    /// is `$name`. It contains the Logos lexer defined inside the module `mode`.
    #[derive(Clone, Debug)]
    pub enum LexerMode<'source> {
      $( $name(logos::Lexer<'source, mode::$name<'source>>), )+
    }

    /// The [logos::Logos::Source] of all lexer modes, usually &str or &[u8].
    /// All lexer modes must have the same source.
    pub type LexerSource<'source> = <
      mode::$init<'source> as logos::Logos<'source>
    >::Source;

    impl<'source> MorphingLexer<'source> {
      pub fn new(source: &'source LexerSource<'source>) -> Self {
        use logos::Logos; // enable Logos::lexer()
        MorphingLexer {
          lexer_mode: LexerMode::$init(mode::$init::lexer(source))
        }
      }

      pub fn mode(&'source self) -> &'source LexerMode { &self.lexer_mode }
    }

    impl<'source> Iterator for MorphingLexer<'source> {
      type Item = Result<MorphingToken<'source>, ()>;

      fn next(&mut self) -> Option<Self::Item> {
        match &mut self.lexer_mode {
          $(
            LexerMode::$mode(lexer) => {
              use mode::$mode::*;

              let result = lexer.next();
              log::trace!("lexer.next() {result:?} mode {}", stringify!($mode));

              // lexer_mode is mutable borrowed and to_owned() must be guaranteed
              // to be called only once. This is achieved by else-if-let arms. In
              // a macro it's easier with a dummy arm which never matches.
              if let Some(_) = None::<()> {
                // never gets executed
              }
              $(
                else if let Some(Ok($token)) = result {
                  self.lexer_mode = LexerMode::$target(lexer.to_owned().morph());
                  log::trace!("morphed to {}", stringify!($target));
                }
              )+

              result.map(|token| token.map(MorphingToken::$mode))
            },
          )+
        }
      }
    }

    /// The module for the Logos lexers. Each of them details a lexer mode.
    pub mod mode {
      use super::*;
      use logos::Logos;

     // This nested macro glues together attributes and the enum
      macro_rules! glue {
        (
          $d( #[ $d( $d meta:meta )+ ] )*
          pub enum $d name:ident $d(< $d lt:lifetime >)? {
            $d( $d tt:tt )+
          }
        ) => {
          // Common mandatory attributes to all lexer modes
          #[derive(Logos, Debug, Clone, Copy, Eq, PartialEq)]

          // Common attributes to all lexer nodes
          $( #[ $( $common_meta )+ ] )*

          // Rest of the enum including attributes
          $d ( #[ $d( $d meta )+ ] )*
          pub enum $d name $d(< $d lt >)? {
            $d( $d tt )+
          }
        }
      }

      $(
        glue!{
          // Per lexer mode attributes and doc comments
          $( #[ $( $meta )+ ] )*

          // Each enum creates a Logos lexer usable as a lexer mode
          pub enum $name $(< $lt >)? {
            $( $tt )+
          }
        }
      )+
    }
  }
}

morphing_lexer! {
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

fn main() {
  let mut lex_init = mode::Init::lexer(r#""\""#.as_bytes());
  let token = lex_init.next();
  assert_eq!(token, Some(Ok(StartStr)));

  let mut lex_str = lex_init.morph::<mode::Str>();
  let token = lex_str.next();
  assert_eq!(token, Some(Ok(Esc(b"\\\""))));
}
