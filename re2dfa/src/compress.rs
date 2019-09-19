use crate::dfa::Dfa;
use bitset::traits::*;
use std::borrow::Borrow;

pub fn ec_of_dfas<D: Borrow<Dfa>>(dfas: &[D]) -> [u8; 256] {
  // 256 bitsets, all with size = 256
  // edge[i][j] (bitset notation) == true <=> i and j cannot use the same char to represent
  let mut edges = [[0u32; 8]; 256];
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

  let mut ec = [0; 256];

  let mut vis = [false; 256];
  fn dfs(edges: &[[u32; 8]; 256], ec: &mut [u8; 256], vis: &mut [bool; 256], x: u8, id: u8) {
    let x = x as usize;
    if vis[x] { return; }
    vis[x] = true;
    ec[x] = id;
    for i in 0..x {
      if !edges[x].as_ref().bsget(i) {
        dfs(edges, ec, vis, i as u8, id);
      }
    }
  }

  let mut id = 0;
  for ch in (0..=255).rev() {
    if !vis[ch as usize] {
      dfs(&edges, &mut ec, &mut vis, ch, id);
      id += 1;
    }
  }

  ec
}

// maybe I will implement dfa table compression in the future?
// there is hardly any documentation about how lex/flex do dfa table compression...