use crate::nfa::Nfa;
use crate::bitset::BitSet;
use print::{IndentPrinter, pretty_chs_display};
use std::collections::{HashMap, HashSet, VecDeque};

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
  pub fn from_nfa(nfa: &Nfa, id: u32) -> Dfa {
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
      nodes.push((if cur.test(nfa.nodes.len() - 1) { Some(id) } else { None }, link));
    }
    Dfa { nodes }
  }

  pub fn print_dot(&self) -> String {
    let mut p = IndentPrinter::new();
    p.ln("digraph g {").inc();
    for (idx, node) in self.nodes.iter().enumerate() {
      let mut outs = HashMap::new();
      for (&k, &out) in &node.1 {
        outs.entry(out).or_insert_with(|| Vec::new()).push(k);
      }
      // just make the graph look beautiful...
      for (out, mut edge) in outs {
        edge.sort_unstable();
        p.ln(format!(r#"{} -> {} [label="{}"];"#, idx, out, pretty_chs_display(&edge)));
      }
      match node.0 {
        Some(id) => p.ln(format!(r#"{}[shape=doublecircle, label="{}\nacc:{}"]"#, idx, idx, id)),
        None => p.ln(format!(r#"{}[shape=circle, label="{}"]"#, idx, idx)),
      };
    }
    p.dec().ln("}");
    p.finish()
  }

  pub fn minimize(&self) -> Dfa {
    let n = self.nodes.len();
    let mut rev_edges = vec![HashMap::new(); n];
    for (i, (_, edges)) in self.nodes.iter().enumerate() {
      for (&k, &out) in edges {
        rev_edges[out as usize].entry(k).or_insert_with(|| Vec::new()).push(i as u32);
      }
    }
    let mut dp = BitSet::new(n * n); // only access lower part
    let mut q = VecDeque::new();
    for (i, (id1, _)) in self.nodes.iter().enumerate() {
      for (j, (id2, _)) in self.nodes.iter().take(i).enumerate() {
        if id1 != id2 {
          unsafe {
            dp.set_unchecked(i * n + j);
          }
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
              let (ii, jj) = if ii < jj { (jj, ii) } else { (ii, jj) };
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
//    you can remove the comments and check the equivalent relation
//    println!();
//    for i in 0..n {
//      print!("{} ", i);
//      for j in 0..n {
//        if dp.test(i * n + j) {
//          print!("X ");
//        } else {
//          print!("  ");
//        }
//      }
//      println!()
//    }
//    print!("  ");
//    for i in 0..n {
//      print!("{} ", i);
//    }
//    println!("\n");

    const INVALID: u32 = !1 + 1;
    let mut vis = vec![INVALID; n];
    let mut q = VecDeque::new();
    let mut id2old = Vec::new();
    let dead_node = (0..n).find(|&i| {
      if self.nodes[i].0.is_some() {
        return false;
      } else {
        for (_, &out) in &self.nodes[i].1 {
          if out != i as u32 {
            return false;
          }
        }
        return true;
      }
    });
    for i in (0..n).rev() {
      if Some(i) != dead_node && vis[i] == INVALID {
        let id = id2old.len() as u32;
        vis[i] = id;
        q.push_back(i);
        id2old.push(vec![i as u32]);
        while let Some(cur) = q.pop_front() {
          for j in 0..cur {
            if vis[j] == INVALID {
              unsafe {
                if !dp.test_unchecked(cur * n + j) {
                  vis[j] = id;
                  q.push_back(j);
                  id2old.get_unchecked_mut(id as usize).push(j as u32);
                }
              }
            }
          }
        }
      }
    }
    id2old.reverse();

    let new_len = id2old.len() as u32;
    let mut nodes = Vec::new();
    for old in id2old {
      let mut link = HashMap::new();
      let id = self.nodes[old[0] as usize].0;
      for o in old {
        for (&k, &out) in &self.nodes[o as usize].1 {
          if Some(out as usize) != dead_node {
            link.insert(k, new_len - 1 - vis[out as usize]);
          }
        }
      }
      nodes.push((id, link));
    }
    Dfa { nodes }
  }

  // basically it is just like turning an nfa to an dfa
  pub fn merge(xs: &[Dfa]) -> Dfa {
    let mut alphabet = HashSet::new();
    for dfa in xs {
      for node in &dfa.nodes {
        for (&k, _) in &node.1 {
          alphabet.insert(k);
        }
      }
    }
    let alphabet = alphabet.into_iter().collect::<Vec<_>>();
    let mut n_nodes = Vec::new();
    let mut accept = HashMap::new();
    let mut begs = Vec::new();
    for dfa in xs {
      let len = n_nodes.len() as u32;
      begs.push(len);
      for (idx, node) in dfa.nodes.iter().enumerate() {
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
        let id = ss.len() as u32;
        let id = *ss.entry(tmp.clone()).or_insert_with(|| {
          q.push_back(tmp.clone());
          id
        });
        link.insert(k, id);
      }
      const INVALID: u32 = !1 + 1;
      let mut min_id = INVALID;
      for i in 0..n_nodes.len() {
        if unsafe { cur.test_unchecked(i) } {
          if let Some(&id) = accept.get(&(i as u32)) {
            min_id = min_id.min(id);
          }
        }
      }
      nodes.push((if min_id != INVALID { Some(min_id) } else { None }, link));
    }
    Dfa { nodes }
  }
}
