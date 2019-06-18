#[macro_use]
extern crate smallvec;

mod printer;
mod re;
mod nfa;
mod dfa;
mod bitset;

fn main() {
  let re = re::parse("(a|b|c|d)*").unwrap();
  let nfa = nfa::Nfa::from_re(&re);
  println!("{:?}", nfa);
  let dfa = dfa::Dfa::from_nfa(&nfa);
//  println!("{:?}", dfa);

  use std::fs::File;
  use std::io::prelude::*;
  let mut f = File::create("nfa.dot").unwrap();
  f.write(dfa.print_dot().as_bytes()).unwrap();
//  println!("{:?}", parse("(ab|cd)+"));
}
