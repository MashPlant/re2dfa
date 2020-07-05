use crate::{Nfa, Dfa};
use std::fmt::Write;
use std::collections::HashMap;
use pretty_u8::pretty_u8;

fn pretty_u8s(chs: &[u8]) -> String {
  let mut s = String::new();
  let mut i = 0;
  while i < chs.len() {
    let mut j = i;
    while j + 1 < chs.len() && chs[j + 1] == chs[j] + 1 { j += 1; }
    if j <= i + 1 {
      for i in i..=j { let _ = write!(s, "{}, ", pretty_u8(chs[i])); }
    } else {
      let _ = write!(s, "{}-{}, ", pretty_u8(chs[i]), pretty_u8(chs[j]));
    }
    i = j + 1;
  }
  s.pop();
  s.pop();
  s
}

impl Nfa {
  pub fn print_dot(&self) -> String {
    let mut s = "digraph g {\n".to_owned();
    for (idx, node) in self.nodes.iter().enumerate() {
      let mut outs = HashMap::new();
      for (&k, out) in node {
        for &out in out {
          if let Some(k) = k {
            outs.entry(out).or_insert_with(Vec::new).push(k);
          } else {
            let _ = writeln!(s, r#"{} -> {} [label="ε"];"#, idx, out);
          }
        }
      }
      for (out, mut edge) in outs {
        edge.sort_unstable();
        let _ = writeln!(s, r#"{} -> {} [label="{}"];"#, idx, out, pretty_u8s(&edge));
      }
      let _ = writeln!(s, r#"{}[shape={}, label="{0}"]"#, idx, if idx == self.nodes.len() - 1 { "doublecircle" } else { "circle" });
    }
    s.push('}');
    s
  }
}

impl Dfa {
  pub fn print_dot(&self) -> String {
    let mut s = "digraph g {\n".to_owned();
    for (idx, node) in self.nodes.iter().enumerate() {
      let mut outs = HashMap::new();
      for (&k, &out) in &node.1 {
        outs.entry(out).or_insert_with(Vec::new).push(k);
      }
      for (out, mut edge) in outs {
        edge.sort_unstable();
        let _ = writeln!(s, r#"{} -> {} [label="{}"];"#, idx, out, pretty_u8s(&edge));
      }
      match node.0 {
        Some(id) => { let _ = writeln!(s, r#"{}[shape=doublecircle, label="{}\nacc:{}"]"#, idx, idx, id); }
        None => { let _ = writeln!(s, r#"{}[shape=circle, label="{}"]"#, idx, idx); }
      };
    }
    s.push('}');
    s
  }
}
