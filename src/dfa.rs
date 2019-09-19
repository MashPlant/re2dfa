use crate::nfa::Nfa;
use bitset::BitSet;
use std::collections::{HashMap, HashSet, VecDeque};
use std::borrow::Borrow;
use std::fmt::Write;

type DfaNode = HashMap<u8, u32>;

// nodes[i].0 stands for node state(whether is terminal, and which nfa it belongs)
#[derive(Debug)]
pub struct Dfa {
  pub nodes: Vec<(Option<u32>, DfaNode)>,
}

impl Dfa {
  fn e_close(bs: &mut BitSet, nfa: &Nfa) {
    let mut changed = true;
    while changed {
      changed = false;
      for (i, edges) in nfa.nodes.iter().enumerate() {
        unsafe {
          if bs.test_unchecked(i) {
            if let Some(eps) = edges.get(&None) {
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
  // the generated dfa contains a dead state, which will be of help in minimizing it
  pub fn from_nfa(nfa: &Nfa, id: u32) -> Dfa {
    let mut alphabet = HashSet::new();
    for edges in &nfa.nodes {
      for (&k, _) in edges {
        alphabet.insert(k);
      }
    }
    alphabet.remove(&None);
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
    while let Some(cur) = q.pop_front() {
      let mut link = HashMap::new();
      for &k in &alphabet {
        let k = k.unwrap(); // it is safe, because we removed `None` in `alphabet`
        tmp.clear_all();
        for (i, edges) in nfa.nodes.iter().enumerate() {
          unsafe {
            if cur.test_unchecked(i) {
              if let Some(outs) = edges.get(&Some(k)) {
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
      nodes.push((if cur.test(nfa.nodes.len() - 1) { Some(id) } else { None }, link));
    }
    Dfa { nodes }
  }

  pub fn print_dot(&self) -> String {
    let mut p = String::new();
    let _ = writeln!(p, "digraph g {{");
    for (idx, node) in self.nodes.iter().enumerate() {
      let mut outs = HashMap::new();
      for (&k, &out) in &node.1 {
        outs.entry(out).or_insert_with(|| Vec::new()).push(k);
      }
      // just make the graph look beautiful...
      for (out, mut edge) in outs {
        edge.sort_unstable();
        let _ = writeln!(p, r#"{} -> {} [label="{}"];"#, idx, out, print::pretty_chs_display(&edge));
      }
      match node.0 {
        Some(id) => { let _ = writeln!(p, r#"{}[shape=doublecircle, label="{}\nacc:{}"]"#, idx, idx, id); }
        None => { let _ = writeln!(p, r#"{}[shape=circle, label="{}"]"#, idx, idx); }
      };
    }
    let _ = writeln!(p, "}}");
    p
  }

  pub fn minimize(&self) -> Dfa {
    let n = self.nodes.len();
    let mut rev_edges = vec![HashMap::new(); n];
    for (i, (_, edges)) in self.nodes.iter().enumerate() {
      for (&k, &out) in edges {
        rev_edges[out as usize].entry(k).or_insert_with(|| Vec::new()).push(i as u32);
      }
    }
    let mut dp = BitSet::new(n * n); // only access upper part, i.e., should guarantee i < j if access it with i * n + j
    let mut q = VecDeque::new();
    for (i, (id1, _)) in self.nodes.iter().enumerate() {
      for (j, (id2, _)) in self.nodes.iter().skip(i).enumerate() {
        let j = i + j; // the real index (compensate for `skip(i)`)
        if id1 != id2 {
          unsafe { dp.set_unchecked(i * n + j); }
          q.push_back((i as u32, j as u32));
        }
      }
    }
    while let Some((i, j)) = q.pop_front() {
      let (rev_i, rev_j) = (&rev_edges[i as usize], &rev_edges[j as usize]);
      for (k_i, out_i) in rev_i {
        if let Some(out_j) = rev_j.get(k_i) {
          for &ii in out_i {
            for &jj in out_j {
              let (ii, jj) = if ii < jj { (ii, jj) } else { (jj, ii) };
              unsafe {
                if !dp.test_unchecked(ii as usize * n + jj as usize) {
                  dp.set_unchecked(ii as usize * n + jj as usize);
                  q.push_back((ii, jj));
                }
              }
            }
          }
        }
      }
    }

    const INVALID: u32 = !0;
    let mut ids = vec![INVALID; n];
    let mut q = VecDeque::with_capacity(n);
    let mut id2old = Vec::with_capacity(n);
    let dead_node = (0..n).find(|&i| {
      // not accept state and no out edge
      self.nodes[i].0.is_none() && self.nodes[i].1.iter().all(|(_, &out)| out == i as u32)
    });
    for i in 0..n {
      if dead_node != Some(i) && ids[i] == INVALID {
        let id = id2old.len() as u32;
        ids[i] = id;
        q.push_back(i);
        let mut old = vec![i as u32];
        while let Some(cur) = q.pop_front() {
          for j in cur..n {
            if ids[j] == INVALID {
              if unsafe { !dp.test_unchecked(cur * n + j) } {
                ids[j] = id;
                q.push_back(j);
                old.push(j as u32);
              }
            }
          }
        }
        id2old.push(old);
      }
    }

    let mut nodes = Vec::new();
    for old in id2old {
      let mut link = HashMap::new();
      let acc = self.nodes[old[0] as usize].0; // they must have the same acc, so pick the acc of 0
      for o in old {
        for (&k, &out) in &self.nodes[o as usize].1 {
          if dead_node != Some(out as usize) {
            link.insert(k, ids[out as usize]);
          }
        }
      }
      nodes.push((acc, link));
    }
    Dfa { nodes }
  }

  // basically it is just like turning an nfa to an dfa
  // the return dfa is minimized and contains no dead state if input `dfas` are all minimized and contain no dead state
  pub fn merge<D: Borrow<Dfa>>(dfas: &[D]) -> Dfa {
    let mut alphabet = HashSet::new();
    for dfa in dfas {
      for node in &dfa.borrow().nodes {
        for (&k, _) in &node.1 {
          alphabet.insert(k);
        }
      }
    }
    let alphabet = alphabet.into_iter().collect::<Vec<_>>();
    let mut n_nodes = Vec::new();
    let mut accept = HashMap::new();
    let mut begs = Vec::new();
    for dfa in dfas {
      let len = n_nodes.len() as u32;
      begs.push(len);
      for (idx, node) in dfa.borrow().nodes.iter().enumerate() {
        if let Some(id) = node.0 {
          accept.insert(idx as u32 + len, id);
        }
        let edges = node.1.iter().map(|(&k, &v)| (k, v + len)).collect::<HashMap<_, _>>();
        n_nodes.push(edges);
      }
    }
    let mut bs = BitSet::new(n_nodes.len());
    for beg in begs {
      unsafe { bs.set_unchecked(beg as usize); }
    }
    let mut ss = HashMap::new();
    let mut q = VecDeque::new();
    ss.insert(bs.clone(), 0);
    q.push_back(bs);
    let mut tmp = BitSet::new(n_nodes.len());
    let mut nodes = Vec::new();
    while let Some(cur) = q.pop_front() {
      let mut link = HashMap::new();
      for &k in &alphabet {
        tmp.clear_all();
        for (i, edges) in n_nodes.iter().enumerate() {
          unsafe {
            if cur.test_unchecked(i) {
              if let Some(&out) = edges.get(&k) {
                tmp.set_unchecked(out as usize);
              }
            }
          }
        }
        if tmp.any() {
          let id = ss.len() as u32;
          let id = *ss.entry(tmp.clone()).or_insert_with(|| {
            q.push_back(tmp.clone());
            id
          });
          link.insert(k, id);
        }
      }
      let acc = (0..n_nodes.len()).filter_map(|i| unsafe {
        accept.get(&(i as u32)).and_then(|x| if cur.test_unchecked(i) { Some(*x) } else { None })
      }).min();
      nodes.push((acc, link));
    }
    Dfa { nodes }
  }
}
