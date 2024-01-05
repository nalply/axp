use crate::PrettyUtf8;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Atom(pub(crate) Vec<u8>);

impl fmt::Display for Atom {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0.pretty_short(f.precision().unwrap_or(0)))
  }
}

impl Atom {
  pub fn new(atom: &[u8]) -> Self {
    Atom(atom.to_vec())
  }

  pub fn format(&self, width: usize) -> String {
    self.0.pretty_short(width)
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
