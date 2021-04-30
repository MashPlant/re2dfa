use std::collections::VecDeque;
use crate::*;

type DfaNode = (Option<u32>, HashMap<u8, u32>);

// nodes[i].0 stands for node state(whether is terminal, and which nfa it belongs)
// a valid Dfa should have nodes.len() >= 1
pub struct Dfa {
  pub nodes: Vec<DfaNode>,
  // all the `u8` keys in `nodes` should be within [0, ec_num)
  pub ec_num: usize,
  // `ec[x] == y` means x is mapped to y in `nodes`
  pub ec: [u8; 256],
}

impl Dfa {
  // the generated dfa contains a dead state, which will eliminated when minimizing it
  pub fn from_nfa(nfa: &Nfa) -> Dfa {
    let (ec_num, nfa_node, nfa_e_close) = (nfa.ec_num, nfa.nodes.as_ptr(), nfa.e_close.as_ptr());
    let elem_len = bitset::bslen(nfa.nodes.len());

    assert!(elem_len != 0 && elem_len * nfa.nodes.len() == nfa.e_close.len());

    let mut tmp = Box::<[u32]>::from(vec![0; elem_len]);
    let mut ss = HashMap::default();
    let mut q = VecDeque::new();

    // eps closure of nfa node 0
    let start = Box::<[u32]>::from(unsafe { std::slice::from_raw_parts(nfa_e_close, elem_len) });
    ss.insert(start.clone(), 0);
    q.push_back(start);

    let mut nodes = Vec::new();
    while let Some(cur) = q.pop_front() {
      let cur = bitset::ibs(&cur);
      let mut link = HashMap::default();
      for k in 0..ec_num {
        bitset::bs(&mut tmp).clear();
        cur.ones(|i| unsafe {
          if let Some(outs) = (*nfa_node.add(i)).edges.get(&(k as u8)) {
            for &out in outs {
              bitset::ubs(&*tmp).or(nfa_e_close.add(out as usize * elem_len), elem_len);
            }
          }
        });
        let id = ss.len() as u32;
        let id = *ss.entry(tmp.clone()).or_insert_with(|| {
          q.push_back(tmp.clone());
          id
        });
        link.insert(k as u8, id);
      }
      let mut id = None;
      cur.ones(|i| unsafe { if id.is_none() { id = (*nfa_node.add(i)).id; } });
      nodes.push((id, link));
    }
    Dfa { nodes, ec_num, ec: nfa.ec }
  }

  pub fn minimize(&mut self) {
    assert!(!self.nodes.is_empty());

    let n = self.nodes.len();
    let mut rev_edges = vec![HashMap::default(); n];
    let rev_edges = rev_edges.as_mut_ptr();
    for (i, (_, edges)) in self.nodes.iter().enumerate() {
      for (&k, &out) in edges {
        unsafe { (*rev_edges.add(out as usize)).entry(k).or_insert(Vec::new()).push(i as u32); }
      }
    }
    let dp = bitset::bsmake(n * n); // only access upper part, i.e., should guarantee i < j if access it with i * n + j
    let dp = unsafe { bitset::ubs(&*dp) };
    let mut q = VecDeque::new();
    for (i, &(id1, _)) in self.nodes.iter().enumerate() {
      for (j, &(id2, _)) in self.nodes.iter().enumerate().skip(i) {
        if id1 != id2 {
          dp.set(i * n + j);
          q.push_back((i as u32, j as u32));
        }
      }
    }
    while let Some((i, j)) = q.pop_front() {
      let (rev_i, rev_j) = unsafe { (&*rev_edges.add(i as usize), &*rev_edges.add(j as usize)) };
      for (k_i, out_i) in rev_i {
        if let Some(out_j) = rev_j.get(k_i) {
          for &i in out_i {
            for &j in out_j {
              let (i, j) = if i < j { (i, j) } else { (j, i) };
              if !dp.get(i as usize * n + j as usize) {
                dp.set(i as usize * n + j as usize);
                q.push_back((i, j));
              }
            }
          }
        }
      }
    }

    const INVALID: u32 = !0;
    let mut ids = vec![INVALID; n];
    let mut id2old = Vec::with_capacity(n);
    let mut q = VecDeque::with_capacity(n);
    // if there is no node, we can't delete this node, this is the requirement of Dfa (nodes.len() >= 1)
    let dead_node = if n == 1 { None } else {
      self.nodes.iter().enumerate().position(|(i, node)|
        node.0.is_none() && node.1.iter().all(|(_, &out)| out == i as u32))
    };
    for i in 0..n {
      unsafe {
        if dead_node != Some(i) && *ids.get_unchecked(i) == INVALID {
          let id = id2old.len() as u32;
          *ids.get_unchecked_mut(i) = id;
          q.push_back(i);
          let mut old = vec![i as u32];
          while let Some(cur) = q.pop_front() {
            for j in cur..n {
              if *ids.get_unchecked(j) == INVALID {
                if !dp.get(cur * n + j) {
                  *ids.get_unchecked_mut(j) = id;
                  q.push_back(j);
                  old.push(j as u32);
                }
              }
            }
          }
          id2old.push(old);
        }
      }
    }

    let mut nodes = Vec::new();
    for old in id2old {
      unsafe {
        let mut link = HashMap::default();
        // they must have the same acc, so pick the acc of old[0]
        let acc = self.nodes.get_unchecked(*old.get_unchecked(0) as usize).0;
        for o in old {
          for (&k, &out) in &self.nodes.get_unchecked(o as usize).1 {
            if dead_node != Some(out as usize) {
              link.insert(k, *ids.get_unchecked(out as usize));
            }
          }
        }
        nodes.push((acc, link));
      }
    }
    self.nodes = nodes;
  }
}
