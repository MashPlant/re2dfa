use crate::nfa::Nfa;
use crate::bitset::BitSet;
use crate::printer::IndentPrinter;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ascii::escape_default;

type DfaNode = HashMap<u8, u32>;

#[derive(Debug)]
pub struct Dfa {
  nodes: Vec<DfaNode>,
  accept: BitSet,
}

impl Dfa {
  fn e_close(bs: &mut BitSet, nfa: &Nfa) {
    let mut changed = true;
    while changed {
      changed = false;
      for (i, edges) in nfa.nodes.iter().enumerate() {
        unsafe {
          if bs.test_unchecked(i) {
            if let Some(eps) = edges.get(&0) {
              for &out in eps {
                changed |= !(bs.test_unchecked(out as usize));
                bs.set_unchecked(out as usize);
              }
            }
          }
        }
      }
    }
  }
}

impl Dfa {
  pub fn from_nfa(nfa: &Nfa) -> Dfa {
    let mut alphabet = HashSet::new();
    for edges in &nfa.nodes {
      for (&k, _) in edges {
        alphabet.insert(k);
      }
    }
    alphabet.remove(&0);
    let alphabet = alphabet.into_iter().collect::<Vec<_>>();
    let mut bs = BitSet::new(nfa.nodes.len());
    bs.set(0);
    Dfa::e_close(&mut bs, nfa);
    let mut ss = HashMap::new();
    let mut q = VecDeque::new();
    ss.insert(bs.clone(), 0);
    q.push_back(bs);
    let mut tmp = BitSet::new(nfa.nodes.len());
    let mut nodes = Vec::new();
    let mut accept = Vec::new();
    while let Some(cur) = q.pop_front() {
      let mut link = HashMap::new();
      for &k in &alphabet {
        tmp.clear_all();
        for (i, edges) in nfa.nodes.iter().enumerate() {
          unsafe {
            if cur.test_unchecked(i) {
              if let Some(outs) = edges.get(&k) {
                for &out in outs {
                  tmp.set_unchecked(out as usize);
                }
              }
            }
          }
        }
        Dfa::e_close(&mut tmp, nfa);
        let id = ss.len() as u32;
        let id = *ss.entry(tmp.clone()).or_insert_with(|| {
          q.push_back(tmp.clone());
          id
        });
        link.insert(k, id);
      }
      nodes.push(link);
      accept.push(cur.test(nfa.nodes.len() - 1));
    }
    Dfa { nodes, accept: BitSet::from_vec(&accept) }
  }

  pub fn print_dot(&self) -> String {
    let mut p = IndentPrinter::new();
    p.ln("digraph g {").inc();
    for (idx, node) in self.nodes.iter().enumerate() {
      for (&k, &out) in node {
        p.ln(format!(r#"{} -> {} [label="{}"];"#, idx, out, (k as char).escape_default()));
      }
      if self.accept.test(idx) {
        p.ln(format!(r#"{}[shape=doublecircle, label="{}"]"#, idx, idx));
      } else {
        p.ln(format!(r#"{}[shape=circle, label="{}"]"#, idx, idx));
      }
    }
    p.dec().ln("}");
    p.finish()
  }

  pub fn minimize(&mut self) {
    unimplemented!()
  }

  pub fn merge(a: &Dfa, b: &Dfa) -> Dfa {
    unimplemented!()
  }
}
