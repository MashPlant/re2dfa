#[macro_use]
extern crate smallvec;

use crate::re::Re;
use crate::dfa::Dfa;
use crate::nfa::Nfa;

mod printer;
mod re;
mod nfa;
mod dfa;
mod bitset;

fn re2dfa(re: &Re, id: u32) -> Dfa {
  Dfa::from_nfa(&Nfa::from_re(re), id).minimize()
}

fn main() {
  let dfa = [
    &re::parse("class").unwrap(),
    &re::parse("int").unwrap(),
    &re::parse("\\d+").unwrap(),
    &re::parse("\\s+").unwrap(),
    &re::parse("(a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z)(_|a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|0|1|2|3|4|5|6|7|8|9)*").unwrap(),
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
