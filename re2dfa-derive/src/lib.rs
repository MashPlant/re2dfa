extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{ItemEnum, Fields, Meta, Lit};
use std::fmt::Write;
use aho_corasick::AhoCorasick;

// this is basically copied from Logos...
// who can teach me how to use a license...
#[proc_macro_derive(Dfa, attributes(re, eps, eof))]
pub fn dfa(input: TokenStream) -> TokenStream {
  let item: ItemEnum = syn::parse(input).expect("#[re] can be only applied to enums");
  let name = item.ident.to_string();

  let mut eps = None;
  let mut eof = None;

  let mut res = Vec::new();

  for variant in &item.variants {
    if variant.discriminant.is_some() {
      panic!("`{}::{}` has a discriminant value set. This is not allowed for Tokens.", name, variant.ident);
    }

    match variant.fields {
      Fields::Unit => {}
      _ => panic!("`{}::{}` has fields. This is not allowed for Tokens.", name, variant.ident),
    }

    let mut it = variant.attrs.iter();
    match it.next() {
      Some(attr) => {
        let attr_name = &attr.path.segments[0].ident;
        let token = &variant.ident;

        if attr_name == "eof" {
          match eof {
            Some(_) => panic!("Only one #[eof] variant can be declared."),
            None => eof = Some(token),
          }
        } else {
          let meta = match attr.parse_meta() {
            Ok(meta) => meta,
            Err(_) => panic!("Couldn't parse attribute: {:?}", attr),
          };
          let lit = match meta {
            Meta::Word(ref ident) if ident == "re" => panic!("Expected #[re = ...], or #[re(...)]"),
            Meta::NameValue(nv) => if nv.ident == "re" { Some(nv.lit) } else { None },
            _ => None,
          };
          if let Some(Lit::Str(re)) = lit {
            match it.next() {
              Some(attr) if attr.path.segments[0].ident == "eps" => {
                // will not be returned (to the parser)
                match eps {
                  Some(_) => panic!("Only one #[eps] variant can be declared."),
                  None => eps = Some(token),
                }
              }
              _ => {}
            }
            res.push((re.value(), token))
          } else {
            panic!("`{}::{}` is not a valid eof/re.", name, variant.ident)
          }
        }
      }
      None => panic!("`{}::{}` is not a valid eof/re.", name, variant.ident)
    }
  }
  // I still don't know how to use quote!, pls someone teach me...
  let template = include_str!("template/template.rs");

  let eof = match eof {
    Some(eof) => eof,
    None => panic!("Missing #[eof] token variant."),
  }.to_string();
  let handle_eps1 = match eps {
    Some(eps) => format!(r#"
    if last_acc != {} {{
      return Some(re2dfa::Token {{ ty: last_acc, piece, line, col }});
    }} else {{
      line = self.cur_line;
      col = self.cur_col;
      last_acc = {};
      state = 0;
      i = 0;
    }}"#, eps, eof),
    None => "return Some(re2dfa::Token { ty: last_acc, piece, line, col });".to_owned(),
  };
  let handle_eps2 = match eps {
    Some(eps) => format!(r#"
    if last_acc != {} {{
      return Some(re2dfa::Token {{ ty: last_acc, piece, line, col }});
    }} else {{
      return Some(re2dfa::Token {{ ty: {}, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col }});
    }}"#, eps, eof),
    None => "return Some(re2dfa::Token { ty: last_acc, piece, line, col });".to_owned(),
  };

  let (dfa, ec) = re2dfa::re2dfa(res.iter().map(|(re, _)| re)).expect("Invalid re.");
  let mut ch2ec = String::new();
  for ch in 0..256 {
    let _ = write!(ch2ec, "{}, ", ec[ch]);
  }
  let u_dfa_size = (match dfa.nodes.len() {
    0..=255 => "u8",
    256..=65535 => "u16",
    _ => "u32",
  }).to_owned();
  let ec_size = (*ec.iter().max().unwrap() + 1).to_string();
  let dfa_size = (dfa.nodes.len()).to_string();
  let mut edge = String::new();
  {
    let mut outs = vec![0; (*ec.iter().max().unwrap() + 1) as usize];
    for (_, edges) in dfa.nodes.iter() {
      for x in &mut outs { *x = 0; }
      for (&k, &out) in edges {
        outs[ec[k as usize] as usize] = out;
      }
      let _ = write!(edge, "{:?}, ", outs);
    }
  }
  let edge = edge;
  let mut acc = String::new();
  for &(acc_, _) in &dfa.nodes {
    match acc_ {
      Some(acc_) => { let _ = write!(acc, "{}, ", res[acc_ as usize].1); }
      None => { let _ = write!(acc, "{}, ", eof); }
    }
  }
  let pat = [
    "{{T}}",
    "{{DFA_SIZE}}",
    "{{ACC}}",
    "{{EC}}",
    "{{U_DFA_SIZE}}",
    "{{EC_SIZE}}",
    "{{DFA_EDGE}}",
    "{{EOF}}",
    "{{HANDLE_EPS1}}",
    "{{HANDLE_EPS2}}",
  ];
  let rep = [
    name,
    dfa_size,
    acc,
    ch2ec,
    u_dfa_size,
    ec_size,
    edge,
    eof,
    handle_eps1,
    handle_eps2
  ];
  let ac = AhoCorasick::new(&pat);
  let result = ac.replace_all(template, &rep);
  result.parse().unwrap()
}