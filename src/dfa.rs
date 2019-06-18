use crate::nfa::Nfa;
use std::collections::HashMap;

type DfaNode = HashMap<u8, u32>;

pub struct Dfa {
  nodes: Vec<DfaNode>,
}

impl Dfa {
  pub fn from_nfa(nfa: &Nfa) -> Dfa {
    unimplemented!()
  }

  pub fn merge(a: &Dfa, b: &Dfa) -> Dfa {
    unimplemented!()
  }
}
