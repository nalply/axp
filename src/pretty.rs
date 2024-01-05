pub trait PrettyUtf8 {
  /// Pretty print a byte slice (see [pretty_utf8_shorten()])
  fn pretty(&self) -> String;

  /// Pretty print and shorten a byte slice
  /// - shorten to width with ellipsis in the middle
  /// - escape invalid utf8 as `\Uhh..hh;`
  /// - escape control codes and whitespace (except space and newline) as one of:
  ///   `\xhh`, `\Xhhh;`, `\Xhhhh;` or `\Xhhhhh;`
  ///
  /// ```
  /// # use axp::pretty::PrettyUtf8;
  /// let s = b"012\x01456789\xff";
  /// assert_eq!("012\\x01456789\\Uff;", s.pretty());
  /// assert_eq!("012\u{fffd}456789\u{fffd}", s.pretty_short(Some(10)));
  /// assert_eq!("012⠤9\u{fffd}", s.pretty_short(Some(5)));
  /// assert_eq!("0⠤\u{fffd}", s.pretty_short(Some(2)));
  /// assert_eq!("0⠤", s.pretty_short(Some(1)));
  /// ```
  fn pretty_short(&self, width: usize) -> String;
}

impl PrettyUtf8 for [u8] {
  fn pretty(&self) -> String {
    pretty(self, 0)
  }

  fn pretty_short(&self, width: usize) -> String {
    pretty(self, width)
  }
}

// Size considerations:
//   - valid item: max size (as escaped char as \U{hhhhh}): 8 bytes
//   - invalid item: byte always escaped a two hex digits: 2 bytes
//   - invalid_coalesced: arbitrarily long sequence of hex digits
// todo: just used String for everything, it's easier
#[derive(Clone, Debug, Eq, PartialEq)]
struct Output {
  contents: String,
  input_len: usize,
  valid_utf8: bool,
}

impl Output {
  fn char_count(&self) -> usize {
    self.contents.chars().count()
  }
}

struct OutputIterator<'b>(&'b [u8]);

const SAFE_UTF8: &str = "unexpected: already validated as utf8";
const NON_EMPTY: &str = "unexpected: already made sure not empty";

impl<'b> Iterator for OutputIterator<'b> {
  type Item = Output;

  fn next(&mut self) -> Option<Self::Item> {
    fn ascii_x_esc(c: char) -> bool {
      c.is_ascii_control()
    }

    fn x_esc(c: char) -> bool {
      c.is_control() && c.is_whitespace() && c > '\u{ff}'
    }

    // read valid char, escape and unshift from bytes then return Output item
    fn valid_char(bytes: &mut &[u8], n: usize) -> Output {
      let c = std::str::from_utf8(&bytes[..n])
        .expect(SAFE_UTF8)
        .chars()
        .next()
        .expect(NON_EMPTY);
      let input_len = c.len_utf8();
      *bytes = &bytes[input_len..];

      let contents = match c {
        '\\' => r"\\".to_owned(),
        '\r' => r"\r".to_owned(),
        '\n' => r"\n".to_owned(),
        '\t' => r"\t".to_owned(),
        '\0' => r"\0".to_owned(),
        c if ascii_x_esc(c) => format!("\\x{:02x}", c as u32),
        c if x_esc(c) => format!("\\X{:x};", c as u32),
        c => c.to_string(),
      };
      Output { contents, input_len, valid_utf8: true }
    }

    if self.0.is_empty() {
      return None;
    }

    let n = self.0.len().min(4);
    let slice = std::str::from_utf8(&self.0[..n]);

    Some(match slice {
      Ok(_) => valid_char(&mut self.0, n),
      Err(err) => {
        let valid_len = err.valid_up_to();
        if valid_len > 0 {
          valid_char(&mut self.0, valid_len.min(4))
        } else {
          let contents = format!("{:02x}", self.0[0]);
          self.0 = &self.0[1..];
          Output { contents, input_len: 1, valid_utf8: false }
        }
      }
    })
  }
}

fn coalesced(output: &[Output]) -> String {
  let mut invalid = false;
  let mut result = String::new();
  for item in output {
    if item.valid_utf8 && invalid {
      invalid = false;
      result.push(';');
    } else if !item.valid_utf8 && !invalid {
      invalid = true;
      result.push_str("\\U");
    }
    result.push_str(&item.contents);
  }
  result
}

pub fn pretty(input: &[u8], width: usize) -> String {
  let width = match width {
    1..=6 => 6,
    _ => width,
  };

  let shortened = width > 0;
  let width2 = width / 2;

  // The first part (or the main part if not shortened)
  let mut output = Vec::new();
  let mut char_count = 0;
  let mut part1_len = 0;
  for item in OutputIterator(input) {
    if shortened && char_count >= width2 {
      break;
    }
    char_count += item.char_count();
    part1_len += item.input_len;
    output.push(item);
  }
  let mut pretty = coalesced(&output);

  // eprint!("w{width} {width2} cc{char_count} i{part1_len} {}", input.len());

  if shortened {
    // Estimate where the second part will start. It's tricky because we don't
    // know the bytes and we don't want to read the whole byte slice, it might
    // be very long. We go backwards from the end of the slice, but this is
    // tricky too because UTF-8... Let's try.

    // One to four bytes get converted to: a char, or be escaped: \xhh, \Xhhh;
    // or \u{hhhhh}. A special case are invalid bytes, sequences of them get
    // coalesced, like this: \Uffffff;
    //
    // The most cautious case to go back as far as neccessary is assuming that
    // all bytes are four-byte chars. To have `width2` chars, we need to go back
    // four times of that. The worst that would happen is that invalid bytes and
    // ASCII control codes strictly alternate, then in that case one byte
    // gets expanded to five chars on average. But we assume that the shortener
    // will work with a few dozen chars at most, so let's discard a few dozen
    // `Output` values, that's not too bad.
    let start2 = input.len().saturating_sub(4 * width2).max(part1_len);
    let output = OutputIterator(&input[start2..]).collect::<Vec<_>>();
    let output_count = output.len();
    let width2 = width2 + (width % 2) - 1;
    // eprint!(" s{start2} w{width2} o{output_count}");

    // Now reverse and count the chars
    let mut char_count = 0;
    let mut part2_len = 0;
    let shortened_count = output
      .iter()
      .rev()
      .take_while(|&item| {
        char_count += item.char_count();
        part2_len += item.input_len;
        char_count <= width2
      })
      .count();

    let start2 = output_count - shortened_count;
    let consumed = part1_len + part2_len >= input.len();
    let start2 = if consumed { 0 } else { start2 };
    // eprint!(" s{shortened_count} i{part2_len} s{start2}");

    // If all bytes both from part1 and part2 are consumed omit gap indicator
    if !consumed {
      pretty.push('⠤');
    }

    pretty.push_str(&coalesced(&output[start2..]))
  }

  // eprintln!();

  pretty
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn utf8_pretty() {
    assert_eq!(b"a".pretty(), "a");
    assert_eq!("ä".as_bytes().pretty(), "ä");
    assert_eq!(b" \n".pretty(), " \\n");
    assert_eq!(b"\0\x01\x07\x13\\\x1f".pretty(), r"\0\x01\x07\x13\\\x1f");
    assert_eq!(b"ASCII text\tand tab".pretty(), r"ASCII text\tand tab");
    assert_eq!(b"not utf8\xf0\x80-".pretty(), r"not utf8\Uf080;-");
    assert_eq!(b"abcd\x00ef\xfegh".pretty(), r"abcd\0ef\Ufe;gh");

    let s = ["abcdefghijklmn", "öde Scheiße 💩 été à Li 李"];
    let t = ["abc⠤mn", "öde⠤ 李"];
    let b = s.iter().map(|s| s.as_bytes()).collect::<Vec<_>>();
    for i in 0..s.len() {
      assert_eq!(b[i].pretty_short(0), s[i], "string {i} iteration 0");
      for j in 1..7 {
        assert_eq!(b[i].pretty_short(j), t[i], "string {i} iteration {j}");
      }
    }

    assert_eq!(b[0].pretty_short(7), "abc⠤lmn");
    assert_eq!(b[0].pretty_short(8), "abcd⠤lmn");
    assert_eq!(b[0].pretty_short(9), "abcd⠤klmn");
    assert_eq!(b[0].pretty_short(10), "abcde⠤klmn");
    assert_eq!(b[0].pretty_short(11), "abcde⠤jklmn");
    assert_eq!(b[0].pretty_short(12), "abcdef⠤jklmn");
    assert_eq!(b[0].pretty_short(13), "abcdef⠤ijklmn");
    assert_eq!(b[0].pretty_short(14), "abcdefghijklmn");
    assert_eq!(b[1].pretty_short(10), "öde S⠤Li 李");
    assert_eq!(b[1].pretty_short(11), "öde S⠤ Li 李");
    assert_eq!(b[1].pretty_short(12), "öde Sc⠤ Li 李");
    assert_eq!(b[1].pretty_short(13), "öde Sc⠤à Li 李");
    assert_eq!(b[1].pretty_short(14), "öde Sch⠤à Li 李");
    assert_eq!(b[1].pretty_short(15), "öde Sch⠤ à Li 李");
    assert_eq!(b[1].pretty_short(16), "öde Sche⠤ à Li 李");
    assert_eq!(b[1].pretty_short(17), "öde Sche⠤é à Li 李");
    assert_eq!(b[1].pretty_short(18), "öde Schei⠤é à Li 李");
    assert_eq!(b[1].pretty_short(19), "öde Schei⠤té à Li 李");
    assert_eq!(b[1].pretty_short(20), "öde Scheiß⠤té à Li 李");
    assert_eq!(b[1].pretty_short(21), "öde Scheiß⠤été à Li 李");
    assert_eq!(b[1].pretty_short(22), "öde Scheiße⠤été à Li 李");
    assert_eq!(b[1].pretty_short(23), "öde Scheiße⠤ été à Li 李");
    assert_eq!(b[1].pretty_short(24), "öde Scheiße 💩 été à Li 李");
    assert_eq!(b[1].pretty_short(25), "öde Scheiße 💩 été à Li 李");
  }

  #[test]
  fn a_test() {
    eprintln!("{}", "öde Scheiße 💩 été à Li 李".as_bytes().pretty_short(24));
  }

  #[test]
  fn fuzzy_pretty() {
    // run many times to see whether panics are triggered
    for _ in 0..1000 {
      let s = rand_bytes();
      let bytes = s.as_slice();
      bytes.pretty();

      let n = bytes.len() as u8;
      bytes.pretty_short((rand::lcr() % n) as usize);

      if n > 10 {
        bytes.pretty_short((10 + rand::lcr() % (n - 10)) as usize);
      }
    }
    let _ = b"asdf".pretty();
  }

  fn rand_bytes() -> Vec<u8> {
    let n = rand::lcr();
    let mut v = Vec::with_capacity(n as usize);
    for _ in 0..n {
      v.push(rand::lcr())
    }
    v
  }

  mod rand {
    use std::num::Wrapping;
    use std::sync::atomic::{AtomicU64, Ordering};

    // by Donald Knuth
    const A: Wrapping<u64> = Wrapping(6364136223846793005);
    const B: Wrapping<u64> = Wrapping(1442695040888963407);

    static SEED: AtomicU64 = AtomicU64::new(42);

    pub fn lcr() -> u8 {
      let x = (A * Wrapping(SEED.load(Ordering::Relaxed)) + B).0;
      SEED.store(x, Ordering::Relaxed);
      x as u8
    }
  }
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
