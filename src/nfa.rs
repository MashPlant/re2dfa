use smallvec::{SmallVec, smallvec};
use crate::*;

#[derive(Debug)]
pub struct NfaNode {
  pub id: Option<u32>,
  pub eps_edges: SmallVec<[u32; 4]>,
  // pub e_close: BitSet,
  pub edges: HashMap<u8, SmallVec<[u32; 4]>>,
}

impl NfaNode {
  fn new(id: Option<u32>, eps_edges: SmallVec<[u32; 4]>, edges: HashMap<u8, SmallVec<[u32; 4]>>) -> NfaNode {
    NfaNode { id, eps_edges, edges }
  }
}

// start state should be 0, a valid Nfa should have nodes.len() >= 1
pub struct Nfa {
  pub nodes: Vec<NfaNode>,
  pub e_close: Box<[u32]>,
  pub ec_num: usize,
  pub ec: [u8; 256],
}

impl Nfa {
  pub fn from_re<'a>(re: impl IntoIterator<Item=&'a [u8]>) -> Result<Nfa, (usize, String)> {
    let mut buf = Vec::new();
    for (id, re) in re.into_iter().enumerate() {
      match parse(re) {
        Ok(re) => buf.push(re),
        Err(err) => return Err((id, err)),
      }
    }
    Ok(Nfa::from_re1(&buf))
  }

  pub fn from_re1(re: &[Re]) -> Nfa {
    let mut edges = [[0u32; 8]; 256];
    for re in re {
      unsafe fn dfs(edges: &mut [[u32; 8]; 256], re: &Re) {
        match re {
          Eps => {}
          &Ch(x) => {
            let x = x as usize;
            for i in 0..x { bitset::ubs(edges.get_unchecked(i)).set(x); }
            for i in x..256 { bitset::ubs(edges.get_unchecked(x)).set(i); }
          }
          Concat(x) | Disjunction(x) => for x in x.iter() { dfs(edges, x); }
          DisjunctionCh(x) => {
            let ubs = bitset::ubs(x.as_ref());
            bitset::ibs(x.as_ref()).ones(|x| {
              for i in 0..x { if !ubs.get(i) { bitset::ubs(edges.get_unchecked(i)).set(x); } }
              for i in x..256 { if !ubs.get(i) { bitset::ubs(edges.get_unchecked(x)).set(i); } }
            })
          }
          Kleene(x) => dfs(edges, x),
        }
      }
      unsafe { dfs(&mut edges, re); }
    }
    let (ec_num, ec) = {
      let mut vis = [false; 256];
      unsafe fn dfs(edges: &[[u32; 8]; 256], ec: &mut [u8; 256], vis: &mut [bool; 256], ch: usize, id: usize) {
        let v = vis.get_unchecked_mut(ch);
        if *v { return; }
        *v = true;
        *ec.get_unchecked_mut(ch) = id as u8;
        for i in ch..256 {
          if !bitset::ubs(edges.get_unchecked(ch)).get(i as usize) {
            dfs(edges, ec, vis, i, id);
          }
        }
      }

      let mut ec_num = 0;
      let mut ec = [0; 256];
      for ch in 0..256 {
        if !vis[ch as usize] {
          unsafe { dfs(&edges, &mut ec, &mut vis, ch, ec_num); }
          ec_num += 1;
        }
      }
      (ec_num, ec)
    };

    let mut nfa = Nfa { nodes: vec![NfaNode::new(None, SmallVec::new(), HashMap::default())], e_close: [].into(), ec_num, ec };
    for (id, re) in re.iter().enumerate() {
      let old_len = nfa.nodes.len();
      nfa.generate(&re, Some(id as u32));
      nfa.nodes[0].eps_edges.push(old_len as u32);
    }
    nfa.compute_e_close();
    nfa
  }

  // a modified version of Thompson construction, remove some useless state
  // the nfa generated from `generate(re, Some(id))` always start at state 0, and accept at state `nodes.len() - 1`
  fn generate(&mut self, re: &Re, id: Option<u32>) {
    let start = self.nodes.len();
    match re {
      Re::Eps => self.nodes.push(NfaNode::new(None, smallvec![start as u32 + 1], HashMap::default())),
      &Re::Ch(c) => {
        let mut edges = HashMap::default();
        edges.insert(self.ec[c as usize], smallvec![start as u32 + 1]);
        self.nodes.push(NfaNode::new(None, SmallVec::new(), edges));
      }
      Re::Concat(c) => for sub in c.iter() { self.generate(sub, None); }
      Re::Disjunction(d) => unsafe {
        const END: u32 = !0;
        self.nodes.push(NfaNode::new(None, SmallVec::new(), HashMap::default()));
        for sub in d.iter() {
          let old_len = self.nodes.len();
          self.generate(sub, None);
          let new_len = self.nodes.len();
          for node in self.nodes.get_unchecked_mut(old_len..) {
            for outs in node.edges.values_mut().chain(Some(&mut node.eps_edges)) {
              for out in outs {
                *out = if *out == new_len as u32 { END } else if *out == old_len as u32 { start as u32 } else { *out - 1 };
              }
            }
          }
          let (all0, sub0) = (&mut *self.nodes.as_mut_ptr().add(start),
                              &mut self.nodes.get_unchecked_mut(old_len));
          for (k, outs) in sub0.edges.iter_mut().map(|(&k, v)| (Some(k), v)).chain(Some((None, &mut sub0.eps_edges))) {
            let dst = if let Some(k) = k { all0.edges.entry(k).or_insert(SmallVec::new()) } else { &mut all0.eps_edges };
            for out in outs {
              if *out != END || !dst.contains(&END) { dst.push(*out); }
            }
          }
          self.nodes.remove(old_len as usize);
        }
        let end = self.nodes.len() as u32;
        for node in self.nodes.get_unchecked_mut(start..) {
          for outs in node.edges.values_mut().chain(Some(&mut node.eps_edges)) {
            for out in outs {
              if *out == END { *out = end };
            }
          }
        }
      }
      Re::DisjunctionCh(d) => {
        let mut edges = HashMap::default();
        bitset::ibs(d.as_ref()).ones(|i| {
          edges.insert(unsafe { *self.ec.get_unchecked(i) }, smallvec![start as u32 + 1]);
        });
        self.nodes.push(NfaNode::new(None, SmallVec::new(), edges));
      }
      Re::Kleene(k) => {
        self.generate(k, None);
        let end = self.nodes.len() as u32 + 1;
        unsafe { self.nodes.get_unchecked_mut(start) }.eps_edges.push(end);
        self.nodes.push(NfaNode::new(id, smallvec![start as u32, end], HashMap::default()));
      }
    }
    if id.is_some() {
      self.nodes.push(NfaNode::new(id, SmallVec::new(), HashMap::default()));
    }
  }

  pub fn compute_e_close(&mut self) {
    let len = self.nodes.len();
    let elem_len = bitset::bslen(len);
    self.e_close = vec![0; elem_len * len].into();
    let p = self.e_close.as_mut_ptr();
    for i in 0..len {
      unsafe { bitset::ubs1(p.add(i * elem_len)).set(i); }
    }
    loop {
      let mut changed = false;
      for (i, node) in self.nodes.iter_mut().enumerate().rev() {
        for &out in &node.eps_edges {
          unsafe { changed |= bitset::ubs1(p.add(i * elem_len)).or(p.add(out as usize * elem_len), elem_len); }
        }
      }
      if !changed { break; }
    }
  }
}