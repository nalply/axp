use core::fmt;

// todo think: wrap &'static strs
// todo think: UTF8-niche of 11xxxxxx of the last byte

/// An LStr (LocalSTRing) is a string type with local (stack) storage. It has a
/// maximum length (given by the const generic parameter `N`) not above 255 (the
/// maximum value of `u8`). It is immutable, because being `Copy` and mutable
/// don't mix and also because mutating utf-8 and staying valid is fussy.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct LStr<const N: usize = 24> {
  len: u8,
  buf: [u8; N],
}

impl<const N: usize> LStr<N> {
  /// True if `self` has a length of zero bytes
  pub const fn is_empty(&self) -> bool {
    self.len == 0
  }

  /// The lenght of `self`
  pub const fn len(&self) -> usize {
    self.len as usize
  }

  /// This `LStr`'s contents as a byte slice
  pub const fn as_bytes(&self) -> &[u8] {
    &self.buf
  }

  /// Try to convert a byte slice to an `LStr`.
  ///
  /// ```
  /// # use crate::axp::lstr::{LStr, LStrError::{self, *}};
  /// let s = LStr::try_from("Hello!").unwrap();
  /// assert_eq!(s, "Hello!");
  ///
  /// let too_long = LStr::try_from("x".repeat(25));
  /// assert_eq!(too_long, Err(TooLong { len: 25 }));
  ///
  /// let not_utf8 = LStr::try_from("ok till here\xff");
  /// assert_eq!(not_utf8, Err(NotUtf8 { valid_up_to: 12 })));
  ///```
  pub fn try_from_uf8(bytes: &[u8]) -> Result<Self, LStrError> {
    let len = bytes.len();

    if len > N {
      Err(LStrError::TooLong { len })
    } else if let Err(err) = std::str::from_utf8(bytes) {
      Err(LStrError::NotUtf8 { valid_up_to: err.valid_up_to() })
    } else {
      let mut buf = [0u8; N];
      buf.copy_from_slice(bytes);
      Ok(LStr { len: len as u8, buf })
    }
  }

  pub fn try_from_str(s: &str) -> Result<Self, LStrError> {
    let len = s.len();

    if len > N {
      Err(LStrError::TooLong { len })
    } else {
      let mut buf = [0u8; N];
      buf.copy_from_slice(s.as_bytes());
      Ok(LStr { len: len as u8, buf })
    }
  }
}

impl<const N: usize> TryFrom<&[u8]> for LStr<N> {
  type Error = LStrError;

  fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
    Self::try_from_uf8(bytes)
  }
}

impl<const N: usize> TryFrom<&str> for LStr<N> {
  type Error = LStrError;

  fn try_from(s: &str) -> Result<Self, Self::Error> {
    Self::try_from_str(s)
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LStrError {
  NotUtf8 { valid_up_to: usize },
  TooLong { len: usize },
}

impl std::error::Error for LStrError {}

impl fmt::Display for LStrError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use LStrError::*;
    match self {
      NotUtf8 { valid_up_to } => write!(f, "not utf8 from index {valid_up_to}"),
      TooLong { len } => write!(f, "too long: {len} bytes"),
    }
  }
}

impl<const N: usize> PartialEq<&str> for LStr<N> {
  fn eq(&self, other: &&str) -> bool {
    self.as_bytes() == other.as_bytes()
  }
}

// todo: proc macro for lstr!() which can panic during compile time
// (and also which manages byte slices better)

#[macro_export]
macro_rules! lstr {
  ( $bstr:literal ) => {
    LStr::try_from_utf8($bstr).unwrap()
  };

  ( s $str:literal ) => {
    LStr::try_from_str($str).unwrap()
  };
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
