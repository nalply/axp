#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::{atom, list, Atom, List, Value};

pub fn evaluate_value(value: &Value) -> Value {
  match value {
    Value::Atom(atom) => evaluate(atom.clone(), List::empty()),
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
  PRIMITIVES.get(&atom).map_or(Value::empty(), |primitive| primitive(&args))
}

pub type Primitive = fn(&List) -> Value;

/// Primitive to implement if
///
/// ```
/// # use axp::{List, atom, list};
/// # use axp::primitive::prim_if;
/// let expr_list = List::new(&[atom!(if), atom!(true), atom!(a)]);
/// assert_eq!(prim_if(&expr_list), atom!(a));
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

pub static PRIMITIVES: LazyLock<HashMap<Atom, Primitive>> =
  LazyLock::new(|| {
    let mut map = HashMap::new();

    map.insert(Atom::new(b"if"), prim_if as Primitive);

    map
  });
