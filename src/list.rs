use crate::Item;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct List(pub(crate) Vec<Item>);

impl fmt::Display for List {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.format(f.precision().unwrap_or(0)))
  }
}

impl List {
  /// Create a new List
  ///
  /// ```
  /// # use axp::{List, Item};
  /// let list = List::new([Item::new_atom(b"atom"), Item::new_list([])]);
  /// assert_eq!(format!("{list}"), "(atom ())");
  /// ```
  pub fn new<I: IntoIterator<Item = Item>>(iter: I) -> Self {
    List(iter.into_iter().collect())
  }

  pub fn nil() -> Self {
    Self::new([])
  }

  pub fn first(&self) -> Item {
    if self.is_empty() {
      Item::nil()
    } else {
      self.0[0].clone()
    }
  }

  pub fn tail(&self) -> List {
    if self.is_empty() {
      List::nil()
    } else {
      List::new(self.0[1..].iter().cloned())
    }
  }

  // todo list should be immutable
  pub fn push(&mut self, item: Item) -> &mut Self {
    self.0.push(item);
    self
  }

  pub fn format(&self, width: usize) -> String {
    let list =
      self.0.iter().map(|v| v.format(width)).collect::<Vec<_>>().join(" ");

    format!("({list})")
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn iter(&self) -> impl Iterator<Item = &Item> {
    self.0.iter()
  }
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
