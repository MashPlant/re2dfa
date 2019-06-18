use crate::re::Re;
use crate::printer::IndentPrinter;
use std::collections::HashMap;
use smallvec::SmallVec;

macro_rules! map (
  { $($key:expr => $value:expr),+ } => {{
    let mut m = ::std::collections::HashMap::new();
    $( m.insert($key, $value); )+
    m
  }};
);

type NfaNode = HashMap<u8, SmallVec<[u32; 4]>>;

// start state should be 0, end state should be (nodes.len() - 1) as u32
#[derive(Debug)]
pub struct Nfa {
  pub nodes: Vec<NfaNode>,
}

impl Nfa {
  pub fn from_re(re: &Re) -> Nfa {
    match re {
      Re::Ch(c) => Nfa { nodes: vec![map! { *c => smallvec![1] }, HashMap::new()] },
      Re::Concat(c) => {
        let mut all = Nfa::from_re(&c[0]);
        for mut sub in c.iter().skip(1).map(Nfa::from_re) {
          let len = all.nodes.len() as u32;
          for edges in &mut sub.nodes {
            for (_, outs) in edges {
              for out in outs {
                *out += len;
              }
            }
          }
          all.nodes.last_mut().unwrap().insert(0, smallvec![len]);
          all.nodes.append(&mut sub.nodes);
        }
        all
      }
      Re::Disjunction(d) => {
        let mut all = Nfa { nodes: vec![HashMap::new()] };
        let mut subs = d.iter().map(Nfa::from_re).collect::<Vec<_>>();
        let end = 1 + subs.iter().map(|it| it.nodes.len() as u32).sum::<u32>();
        for sub in &mut subs {
          let len = all.nodes.len() as u32;
          for edges in &mut sub.nodes {
            for (_, outs) in edges {
              for out in outs {
                *out += len;
              }
            }
          }
          all.nodes[0].entry(0).or_insert_with(|| smallvec![]).push(len);
          sub.nodes.last_mut().unwrap().insert(0, smallvec![end]);
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
        all.nodes[0].insert(0, smallvec![1, end]);
        sub.nodes.last_mut().unwrap().insert(0, smallvec![1, end]);
        all.nodes.append(&mut sub.nodes);
        all.nodes.push(HashMap::new());
        all
      }
    }
  }

  pub fn print_dot(&self) -> String {
    let mut p = IndentPrinter::new();
    p.ln("digraph g {").inc();
    for (idx, node) in self.nodes.iter().enumerate() {
      for (&k, outs) in node {
        let k = if k == 0 { "ε".into() } else { (k as char).to_string() };
        for out in outs {
          p.ln(format!(r#"{} -> {} [label="{}"];"#, idx, out, k));
        }
      }
      if idx == self.nodes.len() - 1 {
        p.ln(format!(r#"{}[shape=doublecircle, label="{}"]"#, idx, idx));
      } else {
        p.ln(format!(r#"{}[shape=circle, label="{}"]"#, idx, idx));
      }
    }
    p.dec().ln("}");
    p.finish()
  }
}