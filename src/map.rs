use std::fmt;

use crate::Item;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Map(pub(crate) Vec<(Item, Item)>);

impl fmt::Display for Map {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.format(f.precision().unwrap_or(0)))
  }
}

impl Map {
  pub fn new<I: IntoIterator<Item = (Item, Item)>>(iter: I) -> Self {
    Map(iter.into_iter().collect())
  }

  pub fn push(&mut self, key: Item, value: Item) -> &mut Self {
    self.0.push((key, value));
    self
  }

  pub fn format(&self, width: usize) -> String {
    let entries = self.0.iter().map(|e| format_entry(&e.0, &e.1, width));
    let entries = entries.collect::<Vec<String>>().join(" ");

    format!("({entries})")
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

fn format_entry(key: &Item, value: &Item, width: usize) -> String {
  let key = key.format(width);
  let value = value.format(width);

  format!("{key}: {value}")
}

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
