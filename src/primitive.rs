#![allow(dead_code)]

use crate::pretty::PrettyUtf8;
use crate::{Atom, Item, List};

pub fn evaluate_item(item: &Item) -> Item {
  match item {
    Item::Atom(atom) => evaluate(atom.clone(), List::nil()),
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
  evaluate(operator(&list.first()), list.tail())
}

pub fn evaluate(atom: Atom, args: List) -> Item {
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
/// # use axp::{List, atom, list};
/// # use axp::primitive::*;
/// let expr_list = List::new(&[atom!(if), atom!(true), atom!(a)]);
/// assert_eq!(prim_if(&expr_list), atom!(a));
/// assert_eq!(prim_to_bytes(&expr_list), atom!(b"(if true a)"));
/// ```
pub fn prim_if(args: &List) -> Item {
  if args.first().is_empty() {
    args.tail().tail().first()
  } else {
    args.tail().first()
  }
}

pub fn prim_print(args: &List) -> Item {
  let bytes = &to_bytes(args).pretty();
  print!("{bytes}");
  Item::nil()
}

pub fn prim_to_bytes(args: &List) -> Item {
  Item::Atom(Atom(to_bytes(args)))
}

pub fn to_bytes(args: &List) -> Vec<u8> {
  let mut bytes = Vec::<u8>::new();
  to_bytes_list(&mut bytes, args);
  bytes
}

fn to_bytes_list(bytes: &mut Vec<u8>, args: &List) {
  bytes.push(b'(');
  for item in args.0.iter() {
    to_bytes_item(bytes, item);
    bytes.push(b' ');
  }
  bytes.pop();
  bytes.push(b')');
}

fn to_bytes_item(bytes: &mut Vec<u8>, item: &Item) {
  match item {
    Item::Atom(atom) => bytes.append(&mut atom.0.clone()),
    Item::List(list) => to_bytes_list(bytes, list),
    Item::Map(_map) => todo!(),
  }
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
  primitives![if, print, to_bytes, eval, first, tail, quote]
}

static PRIMITIVES: std::sync::OnceLock<Primitives> = std::sync::OnceLock::new();

// Copyright see AUTHORS & LICENSE; SPDX-License-Identifier: ISC+
