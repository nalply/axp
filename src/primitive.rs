#![allow(dead_code)]

use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::shorten_lossy::shorten_lossy;
use crate::{atom, list, Atom, List, Value};

pub fn evaluate_value(value: &Value) -> Value {
  match value {
    Value::Atom(atom) => evaluate(atom.clone(), List::nil()),
    Value::List(list) => evaluate_list(list),
    Value::Map(_) => value.clone(),
  }
}

pub fn operator(op: &Value) -> Atom {
  match op {
    Value::Atom(atom) => atom.clone(),
    Value::List(list) => operator(&evaluate_list(list)),
    Value::Map(_) => Atom::new(b"map_as_operator"),
  }
}

pub fn evaluate_list(list: &List) -> Value {
  evaluate(operator(&list.first()), list.tail())
}

pub fn evaluate(atom: Atom, args: List) -> Value {
  let primitives = PRIMITIVES.get_or_init(|| define_primitives());
  primitives.get(&atom).map_or(list!(), |primitive| primitive(&args))
}

pub type Primitive = fn(&List) -> Value;

#[allow(non_upper_case_globals)]
pub const prim_eval: Primitive = evaluate_list;

pub fn prim_quote(args: &List) -> Value { Value::List(args.clone()) }

pub fn prim_first(args: &List) -> Value { args.first() }

pub fn prim_tail(args: &List) -> Value { Value::List(args.tail()) }

/// Primitive to implement if
///
/// ```
/// # use axp::{List, atom, list};
/// # use axp::primitive::*;
/// let expr_list = List::new(&[atom!(if), atom!(true), atom!(a)]);
/// assert_eq!(prim_if(&expr_list), atom!(a));
/// assert_eq!(prim_to_bytes(&expr_list), atom!(b"(if true a)"));
/// ```
pub fn prim_if(args: &List) -> Value {
  let args = &args.0;
  if args.is_empty() {
    list!()
  } else if args[0] == atom!(b"") || args[0] == list!() {
    args.get(1).cloned().unwrap_or(list!())
  } else {
    args.get(2).cloned().unwrap_or(list!())
  }
}

pub fn prim_print(args: &List) -> Value {
  let bytes = shorten_lossy(&to_bytes(args), None);
  print!("{bytes}");
  list!()
}

pub fn prim_to_bytes(args: &List) -> Value { Value::Atom(Atom(to_bytes(args))) }

pub fn to_bytes(args: &List) -> Vec<u8> {
  let mut bytes = Vec::<u8>::new();
  to_bytes_list(&mut bytes, args);
  bytes
}

fn to_bytes_list(bytes: &mut Vec<u8>, args: &List) {
  bytes.push(b'(');
  for value in args.0.iter() {
    to_bytes_value(bytes, value);
    bytes.push(b' ');
  }
  bytes.pop();
  bytes.push(b')');
}

fn to_bytes_value(bytes: &mut Vec<u8>, value: &Value) {
  match value {
    Value::Atom(atom) => bytes.append(&mut atom.0.clone()),
    Value::List(list) => to_bytes_list(bytes, list),
    Value::Map(_map) => todo!(),
  }
}

macro_rules! primitives {
  ($($name:ident $(,)?),+) => {
    paste::paste! {
      indexmap::indexmap! {
        $(
          Atom::new(stringify!($name).as_bytes()) =>
            [<prim_ $name>] as Primitive,
       )+
      }
    }
  };
}

pub fn define_primitives() -> IndexMap<Atom, Primitive> {
  primitives![if, print, to_bytes, eval, first, tail, quote]
}

static PRIMITIVES: OnceLock<IndexMap<Atom, Primitive>> = OnceLock::new();
