#[macro_use]
extern crate smallvec;

mod printer;
mod re;
mod nfa;
mod dfa;
mod bitset;

fn main() {
  let re = re::parse("(a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z)(_|a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|A|B|C|D|E|F|G|H|I|J|K|L|M|N|O|P|Q|R|S|T|U|V|W|X|Y|Z|0|1|2|3|4|5|6|7|8|9)*").unwrap();
  let nfa = nfa::Nfa::from_re(&re);
  let dfa = dfa::Dfa::from_nfa(&nfa, 0);
  println!("{}", dfa.nodes.len());
  let dfa_min = dfa.minimize();
  println!("{}", dfa_min.nodes.len());

//  println!("{:?}", dfa);

  use std::fs::File;
  use std::io::prelude::*;
  let mut f = File::create("nfa.dot").unwrap();
  f.write(nfa.print_dot().as_bytes()).unwrap();
  let mut f = File::create("dfa.dot").unwrap();
  f.write(dfa.print_dot().as_bytes()).unwrap();
  let mut f = File::create("dfa_min.dot").unwrap();
  f.write(dfa_min.print_dot().as_bytes()).unwrap();
//  println!("{:?}", parse("(ab|cd)+"));
}
