extern crate proc_macro;
extern crate syn;
extern crate re2dfa;
extern crate aho_corasick;

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
  for ch in 0..128 {
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
  println!("{}", result);
  result.parse().unwrap()

//  let tokens = quote! {
//    #[derive(Debug, Clone, Copy)]
//    pub struct #lexer_name<'a> {
//      pub string: &'a [u8],
//      pub cur_line: u32,
//      pub cur_col: u32,
//    }
//
//    impl<'a> #lexer_name<'a> {
//      pub fn new(string: &[u8]) -> Self {
//        Self {
//          string,
//          cur_line: 1,
//          cur_col: 1,
//        }
//      }
//
//      pub fn next(&mut self) -> Option<Token<'a>> {
//        use TokenType::*;
//        static CH2EC: [u8; 128] = #ch2ec;
//        static EDGE: [[#u_dfa_num; #ec_num]; #dfa_num] = #edge;
//        static ACC: [#name; #dfa_num] = #acc;
//
//        loop {
//          if self.string.is_empty() {
//            return Some(Token { ty: _Eof, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col });
//          }
//          let (mut line, mut col) = (self.cur_line, self.cur_col);
//          let mut last_acc = #eof;
//          let mut state = 0;
//          let mut i = 0;
//          while i < self.string.len() {
//            let ch = unsafe { *self.string.get_unchecked(i) };
//            let &ec = unsafe { CH2EC.get_unchecked((ch & 0x7F) as usize) };
//            let &nxt = unsafe { EDGE.get_unchecked(state as usize).get_unchecked(ec as usize) };
//            let &acc = unsafe { ACC.get_unchecked(nxt as usize) };
//            last_acc = if acc != _Eof { acc } else { last_acc };
//            state = nxt;
//            if nxt == 0 {
//              if last_acc == #eof {
//                return None;
//              } else {
//                let piece = &self.string[..i];
//                self.string = &self.string[i..];
//                #eps_handle1
//              }
//            } else { // continue, eat this char
//              if ch == b'\n' {
//                self.cur_line += 1;
//                self.cur_col = 1;
//              } else {
//                self.cur_col += 1;
//              }
//              i += 1;
//            }
//          }
//          // end of file
//          if last_acc == #eof { // completely dead
//            return None;
//          } else {
//            // exec user defined function here
//            let piece = &self.string[..i];
//            self.string = &self.string[i..];
//            #eps_handle2
//          }
//        }
//    }
//  }
//  };
//  println!("{}", tokens);
//  TokenStream::from(tokens).into()
//  let tokens = quote! {
//        impl ::logos::Logos for #name {
//            type Extras = #extras;
//
//            const SIZE: usize = #size;
//            const ERROR: Self = #name::#error;
//            const END: Self = #name::#end;
//
//            fn lexicon<'lexicon, 'source, Source>() -> &'lexicon ::logos::Lexicon<::logos::Lexer<Self, Source>>
//            where
//                Source: ::logos::Source<'source>,
//                Self: ::logos::source::WithSource<Source>,
//            {
//                use ::logos::internal::LexerInternal;
//                use ::logos::source::Split;
//
//                type Lexer<S> = ::logos::Lexer<#name, S>;
//
//                fn _error<'source, S: ::logos::Source<'source>>(lex: &mut Lexer<S>) {
//                    lex.bump(1);
//
//                    lex.token = #name::#error;
//                }
//
//                #fns
//
//                &[#(#handlers),*]
//            }
//        }
//
//        impl<'source, Source: ::logos::source::#source<'source>> ::logos::source::WithSource<Source> for #name {}
//    };
//
//  TokenStream::from(tokens).into()
}