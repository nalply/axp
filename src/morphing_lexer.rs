/// Generate several Logos lexer enums for lexer modes, a super token over all
/// lexer modes and an iterator that morphs between the lexer nodes as specified
/// in the macro. The lexer modes must all implement Copy, this means for
/// example that the tokens can't contain values that are not Copy, for example
/// String. Also not supported are lexer extras. I gladly admit that this is an
/// overengineered macro for a niche use, and not really well document, but
/// whatever. As long as it works for axp, a tiny lisp and document language...
/// ```
/// axp::morphing_lexer! {
///   @dollar: $;
///   @initial_mode: Init;
///   @morphs: { Init { StartStr => Str } Str { EndStr => Init } }
///   @apply_to_all_lexer_mode_enums: {}
///   pub lexer_mode_enum Init<'s> {
///     #[regex("[ \n\r\t]", logos::skip)]
///     WhiteSpace,
///     #[token("\"")]
///     StartStr,
///     #[regex(r"\w+", |lexer| lexer.slice())]
///     Ident(&'s str),
///   }
///   pub lexer_mode_enum Str<'s> {
///     #[regex(r#"[[^\pC\pZ"][ \t]]+"#, |lexer| lexer.slice())]
///     StrContents(&'s str),
///     #[token("\"")]
///     EndStr,
///   }
/// }
///
/// let lex = MorphingLexer::new("hello \"world\"");
/// let tokens = lex.collect::<Vec<_>>();
/// println!("{tokens:?}");
///
/// use mode::Init::*;
/// use mode::Str::*;
/// use MorphingToken::{Init, Str};
/// assert_eq!(tokens, &[
///   Ok(Init(Ident("hello"))),
///   Ok(Init(StartStr)),
///   Ok(Str(StrContents("world"))),
///   Ok(Str(EndStr)),
/// ]);
/// ```
#[macro_export]
macro_rules! morphing_lexer {
  (
    @dollar: $d:tt;
    @initial_mode: $init:ident;
    @morphs: { $( $mode:ident { $( $token:pat => $target:ident $(,)? ),+ } )+ }
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
    #[allow(dead_code)]
    pub type LexerSource<'s> = <mode::$init<'s> as logos::Logos<'s>>::Source;

    impl<'source> MorphingLexer<'source> {
      /// Create a new MorphingLexer instance starting with lexer mode defined
      /// with &#64;initial_mode: in the macro `morphing_lexer! { ... }`.
      #[allow(dead_code)]
      pub fn new(source: &'source LexerSource<'source>) -> Self {
        use logos::Logos; // enable Logos::lexer()
        MorphingLexer {
          lexer_mode: LexerMode::$init(mode::$init::lexer(source))
        }
      }

      /// The current lexer mode.
      #[allow(dead_code)]
      pub fn mode(&'source self) -> &'source LexerMode { &self.lexer_mode }

      // did not work out, problem with lifetime and syntax...
      // #[allow(dead_code)]
      // pub fn mode_name(&'source self) -> &'static str {
      //   match self.lexer_mode {
      //     $( $name(_) => stringify!($name), )+
      //   }
      // }
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

      // This nested macro glues together attributes and the enum, because first
      // $common_meta is repeated at the wrong level and because second macro-
      // generated attributes can't be put before items because of macro hygiene.
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
          pub enum $name $(< $lt >)? { $( $tt )+ }
        }
      )+
    }
  }
}
