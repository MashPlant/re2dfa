use crate::re::Re;
use std::collections::HashMap;
use smallvec::SmallVec;
use std::fmt::Write;

// Option<u8>, Some for a char, None for eps
type NfaNode = HashMap<Option<u8>, SmallVec<[u32; 4]>>;

// start state should be 0, end state should be (nodes.len() - 1) as u32
// a valid Nfa should have nodes.len() >= 2
#[derive(Debug)]
pub struct Nfa {
  pub nodes: Vec<NfaNode>,
}

impl Nfa {
  // a modified Thompson construction, save some useless state
  // this can tremendously increase the speed of nfa -> dfa(about 10 times)
  pub fn from_re(re: &Re) -> Nfa {
    match re {
      Re::Eps => {
        let mut node0 = HashMap::new();
        node0.insert(None, smallvec![1]); // eps
        Nfa { nodes: vec![node0, HashMap::new()] }
      }
      &Re::Ch(c) => {
        let mut node0 = HashMap::new();
        node0.insert(Some(c), smallvec![1]);
        Nfa { nodes: vec![node0, HashMap::new()] }
      }
      Re::Concat(c) => {
        let mut all = Nfa { nodes: vec![] };
        for mut sub in c.iter().map(Nfa::from_re) {
          sub.nodes.pop();
          let (len, sub_len) = (all.nodes.len() as u32, sub.nodes.len() as u32);
          for edges in &mut sub.nodes {
            for (_, outs) in edges {
              for out in outs {
                if *out == sub_len { // point to old end
                  *out = len + sub_len; // now point to new end
                } else {
                  *out += len;
                }
              }
            }
          }
          all.nodes.append(&mut sub.nodes);
        }
        all.nodes.push(HashMap::new());
        all
      }
      Re::Disjunction(d) => {
        let mut all = Nfa { nodes: vec![HashMap::new()] };
        let mut subs = d.iter().map(Nfa::from_re).collect::<Vec<_>>();
        let end = 1 + subs.iter().map(|it| it.nodes.len() as u32 - 1).sum::<u32>();
        for sub in &mut subs {
          sub.nodes.pop();
          let (len, sub_len) = (all.nodes.len() as u32, sub.nodes.len() as u32);
          for edges in &mut sub.nodes {
            for (_, outs) in edges {
              for out in outs {
                if *out == sub_len { // point to old end
                  *out = end; // now point to new end
                } else {
                  *out += len;
                }
              }
            }
          }
          all.nodes[0].entry(None).or_insert_with(|| smallvec![]).push(len);
          all.nodes.append(&mut sub.nodes);
        }
        all.nodes.push(HashMap::new());
        all
      }
      Re::Kleene(k) => {
        let mut all = Nfa { nodes: vec![HashMap::new()] };
        let mut sub = Nfa::from_re(k);
        let end = 1 + sub.nodes.len() as u32;
        for edges in &mut sub.nodes {
          for (_, outs) in edges {
            for out in outs {
              *out += 1;
            }
          }
        }
        all.nodes[0].insert(None, smallvec![1, end]);
        sub.nodes.last_mut().unwrap().insert(None, smallvec![1, end]);
        all.nodes.append(&mut sub.nodes);
        all.nodes.push(HashMap::new());
        all
      }
    }
  }

  pub fn print_dot(&self) -> String {
    let mut p = String::new();
    let _ = writeln!(p, "digraph g {{");
    for (idx, node) in self.nodes.iter().enumerate() {
      let mut outs = HashMap::new();
      for (&k, out) in node {
        for &out in out {
          if let Some(k) = k {
            outs.entry(out).or_insert_with(|| Vec::new()).push(k);
          } else {
            let _ = writeln!(p, r#"{} -> {} [label="ε"];"#, idx, out);
          }
        }
      }
      // just make the graph look beautiful...
      for (out, mut edge) in outs {
        edge.sort_unstable();
        let _ = writeln!(p, r#"{} -> {} [label="{}"];"#, idx, out, print::pretty_chs_display(&edge));
      }
      if idx == self.nodes.len() - 1 {
        let _ = writeln!(p, r#"{}[shape=doublecircle, label="{}"]"#, idx, idx);
      } else {
        let _ = writeln!(p, r#"{}[shape=circle, label="{}"]"#, idx, idx);
      }
    }
    let _ = writeln!(p, "}}");
    p
  }
}