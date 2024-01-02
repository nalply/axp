use core::fmt;
use indexmap::IndexMap;

use crate::pretty::PrettyUtf8;

/// Create an atom.
///
/// ```
/// # use axp::atom;
/// let atom = atom!(b"atom");
/// assert_eq!(format!("{atom}"), "atom");
/// ```
#[macro_export]
macro_rules! atom {
  ($atom:ident) => {
    $crate::atom!(stringify!($atom).as_bytes())
  };

  ($atom:expr) => {
    $crate::Value::atom($atom)
  };
}

/// Create a list.
///
/// ```
/// # use axp::{atom, list};
/// let list = list!(atom!(b"atom"), list!());
/// assert_eq!(format!("{list}"), "(atom ())");
/// ```
#[macro_export]
macro_rules! list {
  ( $( $value:expr ),* ) => {
    $crate::Value::list(&[ $( $value, )* ])
  }
}

/// Create a map.
///
/// ```
/// # use axp::{atom, list, map};
/// let map = map! {
///   list: list!(atom!(b"list-atom"), list!()),
///   key: atom!(b"value"),
/// };
/// assert_eq!(format!("{map}"), "(list: (list-atom ()) key: value)");
/// ```
#[macro_export]
macro_rules! map {
  ( $( $key:ident: $value:expr, )* ) => {
    $crate::Value::map([
      $( ($crate::Atom::new(stringify!($key).as_bytes()), $value), )*
    ].iter().cloned())
  }
}

/// A value is an atom, a list or a map.
///
/// ```
/// # use axp::Value;
/// let atom = Value::atom(b"atom");
/// assert_eq!(format!("{atom}"), "atom");
/// ```
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
  Atom(Atom),
  List(List),
  Map(Map),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Atom(pub Vec<u8>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Map(pub IndexMap<Atom, Value>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct List(pub Vec<Value>);

impl fmt::Display for Value {
  /// Display a value.
  ///
  /// ```
  /// # use axp::{Value, Atom, List, Map};
  /// let list = Value::list(&[Value::atom(b"a"), Value::list(&[])]);
  /// let map = [
  ///   (Atom::new(b"key"), Value::atom(b"value")),
  ///   (Atom::new(b"list"), list.clone()),
  /// ];
  /// let map = Value::map(map.iter().cloned());
  ///
  /// assert_eq!(format!("{list}"), "(a ())");
  /// assert_eq!(format!("{map}"), "(key: value list: (a ()))");
  /// ```
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let width = f.precision().unwrap_or(0);
    match self {
      Value::Atom(Atom(atom)) => f.write_str(&atom.pretty_short(width)),
      Value::List(List(list)) => f.write_str(&format_list(list, width)),
      Value::Map(Map(map)) => f.write_str(&format_map(map, width)),
    }
  }
}

impl fmt::Display for Atom {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.0.pretty_short(f.precision().unwrap_or(0)))
  }
}

impl fmt::Display for List {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&format_list(&self.0, f.precision().unwrap_or(0)))
  }
}

impl fmt::Display for Map {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&format_map(&self.0, f.precision().unwrap_or(0)))
  }
}

impl Atom {
  pub fn new(atom: &[u8]) -> Self {
    Atom(atom.to_vec())
  }
}

impl List {
  /// Create a new List
  ///
  /// ```
  /// # use axp::{List, Value};
  /// let list = List::new(&[Value::atom(b"atom"), Value::list(&[])]);
  /// assert_eq!(format!("{list}"), "(atom ())");
  /// ```
  pub fn new(list: &[Value]) -> Self {
    List(list.to_vec())
  }

  pub fn nil() -> Self {
    Self::new(&[])
  }

  pub fn first(&self) -> Value {
    if self.0.is_empty() {
      list!()
    } else {
      Value::list(&[self.0[0].clone()])
    }
  }

  pub fn tail(&self) -> List {
    List::new(if self.0.is_empty() { &[] } else { &self.0[1..] })
  }
}

impl Map {
  pub fn new<I: IntoIterator<Item = (Atom, Value)>>(iterable: I) -> Self {
    Map(IndexMap::from_iter(iterable))
  }
}

impl Value {
  fn format(&self, width: usize) -> String {
    match self {
      // Code duplication because I could not find out how to pass Display
      // formatting specifiers like `#` (alternate) down to the enum variants
      // like Atom which is also Display. So both the enum Value and the
      // variants use pretty_short, format_list and format_map.
      // Duplication sites are commented like this: // see Value::fmt
      Value::Atom(Atom(atom)) => atom.pretty_short(width),
      Value::List(List(list)) => format_list(list, width),
      Value::Map(Map(map)) => format_map(map, width),
    }
  }

  pub fn atom(atom: &[u8]) -> Value {
    Value::Atom(Atom::new(atom))
  }

  pub fn list(list: &[Value]) -> Value {
    Value::List(List::new(list))
  }

  pub fn map<I: IntoIterator<Item = (Atom, Value)>>(iterable: I) -> Value {
    Value::Map(Map::new(iterable))
  }
}

fn format_list(list: &[Value], width: usize) -> String {
  let list = list.iter().map(|v| v.format(width)).collect::<Vec<_>>().join(" ");

  format!("({list})")
}

fn format_entry(key: &[u8], value: &Value, width: usize) -> String {
  let key = key.pretty_short(width);
  let value = value.format(width);

  format!("{key}: {value}")
}

fn format_map(map: &IndexMap<Atom, Value>, width: usize) -> String {
  let entries = map.iter().map(|(k, v)| format_entry(&k.0, v, width));
  let entries = entries.collect::<Vec<String>>().join(" ");

  format!("({entries})")
}
