use std::fmt;

fn split_point(width: usize, len: usize) -> (usize, usize) {
  ((width + 1) / 2, len - width / 2)
}

fn parts(data: &[u8], width: Option<usize>, len: usize) -> (&[u8], &[u8]) {
  let width = width.unwrap_or(0);
  let (head, tail) = split_point(width, len);
  (&data[..head], &data[tail..])
}

/// Shorten to width and display (using replacement character)
pub fn shorten_lossy(data: &[u8], width: Option<usize>) -> String {
  let len = data.len();
  let (head, tail) = parts(data, width, len);
  let head = String::from_utf8_lossy(head);
  let tail = String::from_utf8_lossy(tail);
  let data = match width {
    Some(width) if len > width + 1 => format!("{head}⠤{tail}"),
    _ => String::from_utf8_lossy(data).to_string(),
  };
  data.chars().map(|c| if c.is_control() { '\u{fffd}' } else { c }).collect()
}

/// A newtype which displays with shorten_lossy from precision.
///
/// ```
/// # use axp::shorten_lossy::ShortenLossy;
/// // replacement character: vvvv      vvvv
/// let s = ShortenLossy(b"012\x01456789\xff");
/// assert_eq!("012\u{fffd}456789\u{fffd}", format!("{s}"));
/// assert_eq!("012\u{fffd}456789\u{fffd}", format!("{s:.10}"));
/// assert_eq!("012⠤9\u{fffd}", format!("{s:.5}"));
/// assert_eq!("0⠤\u{fffd}", format!("{s:.2}"));
/// assert_eq!("0⠤", format!("{s:.1}"));
/// ```
pub struct ShortenLossy<'a>(pub &'a [u8]);

impl fmt::Display for ShortenLossy<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let width = f.precision();
    if f.alternate() {
      let len = self.0.len();
      let (head, tail) = parts(self.0, width, len);
      match width {
        Some(width) if len > width => write!(f, "{head:?} .. {tail:?}"),
        _ => write!(f, "{:?}", self.0),
      }
    } else {
      write!(f, "{}", shorten_lossy(self.0, width))
    }
  }
}
