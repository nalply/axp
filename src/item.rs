use core::fmt;

use crate::{Atom, List, Map};

/// An item is an atom, a list or a map.
///
/// ```
/// # use axp::Item;
/// let atom = Item::atom(b"atom");
/// assert_eq!(format!("{atom}"), "atom");
/// ```
#[allow(dead_code)]
#[derive(Clone, Eq, PartialEq)]
pub enum Item {
  Atom(Atom),
  List(List),
  Map(Map),
}

impl fmt::Debug for Item {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let width = f.precision().unwrap_or(0);
    match self {
      Item::Atom(atom) => f.write_str(&atom.format(width)),
      Item::List(list) => f.write_str(&list.format(width)),
      Item::Map(map) => f.write_str(&map.format(width)),
    }
  }
}

impl fmt::Display for Item {
  /// Display an item.
  ///
  /// ```
  /// # use axp::{Item, Atom, List, Map};
  /// let list = Item::list(&[Item::atom(b"a"), Item::list(&[])]);
  /// let map = [
  ///   (Atom::new(b"key"), Item::atom(b"item")),
  ///   (Atom::new(b"list"), list.clone()),
  /// ];
  /// let map = Item::map(map.iter().cloned());
  ///
  /// assert_eq!(format!("{list}"), "(a ())");
  /// assert_eq!(format!("{map}"), "(key: item list: (a ()))");
  /// ```
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let width = f.precision().unwrap_or(0);
    match self {
      Item::Atom(atom) => f.write_str(&atom.format(width)),
      Item::List(list) => f.write_str(&list.format(width)),
      Item::Map(map) => f.write_str(&map.format(width)),
    }
  }
}

impl Item {
  pub fn format(&self, width: usize) -> String {
    match self {
      // Code duplication because I could not find out how to pass Display
      // formatting specifiers like `#` (alternate) down to the enum variants
      // like Atom which is also Display. So both the enum Item and the
      // variants use pretty_short, format_list and format_map.
      // Duplication sites are commented like this: // see Item::fmt
      Item::Atom(atom) => atom.format(width),
      Item::List(list) => list.format(width),
      Item::Map(map) => map.format(width),
    }
  }

  pub fn new_atom(atom: &[u8]) -> Self {
    Item::Atom(Atom::new(atom))
  }

  pub fn new_list<'a, I: IntoIterator<Item = &'a Item>>(iter: I) -> Self {
    Item::List(List::new(iter))
  }

  pub fn new_map<'a, I: IntoIterator<Item = &'a [&'a Item]>>(iter: I) -> Self {
    Item::Map(Map::new(iter))
  }

  pub fn nil() -> Item {
    Item::List(List::nil())
  }

  pub fn is_empty(&self) -> bool {
    match self {
      Item::Atom(atom) => atom.is_empty(),
      Item::List(list) => list.is_empty(),
      Item::Map(map) => map.is_empty(),
    }
  }

  pub fn is_atom(&self) -> bool {
    matches!(self, Item::Atom(_))
  }

  pub fn is_list(&self) -> bool {
    matches!(self, Item::List(_))
  }

  pub fn is_map(&self) -> bool {
    matches!(self, Item::Map(_))
  }
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
