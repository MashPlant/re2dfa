use crate::dfa::Dfa;
use bitset::traits::*;
use std::collections::HashMap;
use std::borrow::Borrow;

pub fn ec_of_dfas<D: Borrow<Dfa>>(dfas: &[D]) -> [u8; 256] {
  let mut edges = [[0u32; 8]; 256]; // 256 bitsets, all with size = 256
  let mut dfa_outs = [0; 256];
  for dfa in dfas {
    for (_, dfa_edges) in &dfa.borrow().nodes {
      for x in dfa_outs.iter_mut() { *x = 0; }
      for (&k, &out) in dfa_edges {
        dfa_outs[k as usize] = out + 1; // + 1 to distinguish from 0
      }
      for i in 0..256 {
        for j in 0..i {
          // maybe one of them is 0, means that there no such edge out
          // in that case this 2 edges are also distinguishable
          if dfa_outs[j] != dfa_outs[i] {
            edges[i].as_mut().bsset(j);
            edges[j].as_mut().bsset(i);
          }
        }
      }
    }
  }
  let mut s = HashMap::new();
  let mut which = [0; 256];
  for (i, &x) in edges.iter().enumerate() {
    let id = s.len();
    s.entry(x).or_insert(([0u32; 8], id)).0.as_mut().bsset(i);
  }
  for (_, &(g, id)) in s.iter() {
    for i in 0..256 {
      if g.as_ref().bsget(i) {
        which[i] = id as u8;
      }
//      if (g >> i & 1) != 0 {
//      }
    }
  }
  which
}

// maybe I will implement table compression in the near(?) future