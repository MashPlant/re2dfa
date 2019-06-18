#[macro_use]
extern crate smallvec;

use smallvec::SmallVec;

mod printer;
mod re;
mod nfa;
mod dfa;

fn main() {
  let re = re::parse("(ab|cd)+").unwrap();
  let nfa = nfa::Nfa::from_re(&re);
  print!("{:?}", nfa);
  use std::fs::File;
  use std::io::prelude::*;
  let mut f = File::create("nfa.dot").unwrap();
  f.write(nfa.print_dot().as_bytes()).unwrap();
//  println!("{:?}", parse("(ab|cd)+"));
}
