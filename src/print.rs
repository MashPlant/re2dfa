use pretty_u8::pretty_u8;
use std::fmt::{Display, Formatter, Result};
use crate::{Nfa, Dfa, HashMap};

#[inline(always)]
pub fn fn2display(f: impl Fn(&mut Formatter)) -> impl Display {
  struct S<F>(F);
  impl<F: Fn(&mut Formatter)> Display for S<F> {
    fn fmt(&self, f: &mut Formatter) -> Result {
      // error is impossible in our context, so we don't handle it
      (self.0)(f);
      Ok(())
    }
  }
  S(f)
}

fn pretty_u8s<'a>(chs: &'a [u8]) -> impl Display + 'a {
  fn2display(move |f| {
    let mut i = 0;
    while i < chs.len() {
      let mut j = i;
      while j + 1 < chs.len() && chs[j + 1] == chs[j] + 1 { j += 1; }
      let sep = if i == 0 { "" } else { ", " };
      if j <= i + 1 {
        for i in i..=j { let _ = write!(f, "{}{}", sep, pretty_u8(chs[i])); }
      } else {
        let _ = write!(f, "{}{}-{}", sep, pretty_u8(chs[i]), pretty_u8(chs[j]));
      }
      i = j + 1;
    }
  })
}

fn print_dot<'a, T: 'a, I>(ec_num: usize, ec: &[u8; 256], nodes: &'a [T], node_attr: impl Fn(&'a T) -> (Option<u32>, I) + 'a)
                           -> impl Display + 'a where I: IntoIterator<Item=(Option<u8>, &'a [u32])> {
  let mut rev_ec = vec![vec![]; ec_num];
  for (idx, &ec) in ec.iter().enumerate() {
    rev_ec[ec as usize].push(idx as u8);
  }
  fn2display(move |f| {
    let _ = f.write_str("digraph g {\n");
    for (idx, node) in nodes.iter().enumerate() {
      let mut outs = HashMap::default();
      let (id, edges) = node_attr(node);
      for (k, out) in edges {
        for &out in out {
          if let Some(k) = k {
            outs.entry(out).or_insert(Vec::new()).extend(rev_ec[k as usize].iter());
          } else {
            let _ = writeln!(f, r#"{} -> {} [label="ε"];"#, idx, out);
          }
        }
      }
      for (out, mut edge) in outs {
        edge.sort_unstable();
        let _ = writeln!(f, r#"{} -> {} [label="{}"];"#, idx, out, pretty_u8s(&edge));
      }
      match id {
        Some(id) => { let _ = writeln!(f, r#"{}[shape=doublecircle, label="{0}\nacc:{}"]"#, idx, id); }
        None => { let _ = writeln!(f, r#"{}[shape=circle, label="{0}"]"#, idx); }
      };
    }
    let _ = f.write_str("}");
  })
}

impl Nfa {
  pub fn print_dot<'a>(&'a self) -> impl Display + 'a {
    print_dot(self.ec_num, &self.ec, &self.nodes, |node| {
      (node.id, node.edges.iter().map(|(&k, v)| (Some(k), v.as_ref())).chain(Some((None, node.eps_edges.as_ref()))))
    })
  }
}

impl Dfa {
  pub fn print_dot<'a>(&'a self) -> impl Display + 'a {
    print_dot(self.ec_num, &self.ec, &self.nodes, |(id, edges)| {
      (*id, edges.iter().map(|(&k, v)| (Some(k), std::slice::from_ref(v))))
    })
  }
}
