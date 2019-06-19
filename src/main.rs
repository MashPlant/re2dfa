#[macro_use]
extern crate smallvec;
extern crate nom;
extern crate bitset;
extern crate print;

use crate::re::Re;
use crate::dfa::Dfa;
use crate::nfa::Nfa;

mod printer;
pub mod re;
pub mod nfa;
pub mod dfa;

fn re2dfa(re: &Re, id: u32) -> Dfa {
  Dfa::from_nfa(&Nfa::from_re(re), id).minimize()
}

fn main() {
  let dfa = [
    &re::parse("class").unwrap(),
    &re::parse("int").unwrap(),
    &re::parse("\\d+").unwrap(),
    &re::parse("\\s+").unwrap(),
    &re::parse("[a-zA-Z][_a-zA-Z0-9]*").unwrap(),
  ].iter()
    .enumerate()
    .map(|(idx, re)| re2dfa(re, idx as u32))
    .collect::<Vec<_>>();
  let merged = dfa::Dfa::merge(&dfa).minimize();

  use std::fs::File;
  use std::io::prelude::*;
  let mut f = File::create("merged.dot").unwrap();
  f.write(merged.print_dot().as_bytes()).unwrap();
}
