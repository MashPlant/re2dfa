use crate::dfa::Dfa;
use std::collections::HashMap;
use std::borrow::Borrow;

pub fn ec_of_dfas<D: Borrow<Dfa>>(dfas: &[D]) -> [u8; 128] {
  let mut edges = [0; 128];
  let mut dfa_outs = [0; 128];
  for dfa in dfas {
    for (_, dfa_edges) in &dfa.borrow().nodes {
      for x in dfa_outs.iter_mut() { *x = 0; }
      for (&k, &out) in dfa_edges {
        dfa_outs[k as usize] = out + 1;
      }
      for i in 0..128 {
        for j in 0..i {
          // maybe one of them is 0, means that there no such edge out
          // in that case this 2 edges are also distinguishable
          if dfa_outs[j] != dfa_outs[i] {
            edges[i] |= 1u128 << (j as u128);
            edges[j] |= 1u128 << (i as u128);
          }
        }
      }
    }
  }
  let mut s = HashMap::new();
  let mut which = [0; 128];
  for (i, &x) in edges.iter().enumerate() {
    let id = s.len();
    s.entry(x).or_insert_with(|| (0, id)).0 |= 1u128 << (i as u128);
  }
  for (_, &(g, id)) in s.iter() {
    for i in 0..128 {
      if (g >> i & 1) != 0 {
        which[i] = id as u8;
      }
    }
  }
  which
}

// maybe I will implement table compression in the near(?) future