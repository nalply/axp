#![allow(dead_code)]

use crate::{Atom, Item, List};

pub fn evaluate(item: &Item) -> Item {
  match item {
    Item::Atom(atom) => evaluate_atom(atom.clone(), List::nil()),
    Item::List(list) => evaluate_list(list),
    Item::Map(_) => item.clone(),
  }
}

pub fn operator(op: &Item) -> Atom {
  match op {
    Item::Atom(atom) => atom.clone(),
    Item::List(list) => operator(&evaluate_list(list)),
    Item::Map(_) => Atom::new(b"map_as_operator"),
  }
}

pub fn evaluate_list(list: &List) -> Item {
  evaluate_atom(operator(&list.first()), list.tail())
}

pub fn evaluate_atom(atom: Atom, args: List) -> Item {
  let primitives = PRIMITIVES.get_or_init(|| define_primitives());
  let name: &[u8] = &atom.0;
  primitives.get(name).map_or(Item::nil(), |primitive| primitive(&args))
}

pub type Primitive = fn(&List) -> Item;

#[allow(non_upper_case_globals)]
pub const prim_eval: Primitive = evaluate_list;

pub fn prim_quote(args: &List) -> Item {
  Item::List(args.clone())
}

pub fn prim_first(args: &List) -> Item {
  args.first()
}

pub fn prim_tail(args: &List) -> Item {
  Item::List(args.tail())
}

/// Primitive to implement if
///
/// ```
/// # use axp::parse;
/// # use axp::evaluate;
/// let command = parse(b"if true a").unwrap();
/// assert_eq!(format!("{command}"), "(if true a)");
/// let result = evaluate(&command);
/// assert_eq!(format!("{result}"), "a");
/// ```
pub fn prim_if(args: &List) -> Item {
  if args.first().is_empty() {
    args.tail().tail().first()
  } else {
    args.tail().first()
  }
}

pub fn prim_print(args: &List) -> Item {
  print!("( ");
  for item in args.iter() {
    print!("{item} ");
  }
  print!(")");
  Item::nil()
}

type Primitives = std::collections::HashMap<&'static [u8], Primitive>;

macro_rules! primitives {
  ($($name:ident $(,)?),+) => {{
    let mut map = Primitives::new();

    paste::paste! {
      $(
        let name = stringify!($name);
        let present = map.insert(name.as_bytes(), [<prim_ $name>] as Primitive);
        assert_eq!(present, None, "duplicate key {name}");
      )+
    }

    map
  }};
}

pub fn define_primitives() -> Primitives {
  primitives![if, print, eval, first, tail, quote]
}

static PRIMITIVES: std::sync::OnceLock<Primitives> = std::sync::OnceLock::new();

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
