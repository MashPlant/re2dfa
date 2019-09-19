#[macro_use]
extern crate smallvec;

pub mod compress;
pub mod re;
pub mod nfa;
pub mod dfa;

pub use compress::*;
pub use re::*;
pub use nfa::*;
pub use dfa::*;

pub fn re2dfa<I: IntoIterator<Item=S>, S: AsRef<str>>(res: I) -> Result<(Dfa, [u8; 256]), (usize, String)> {
  let mut dfas = Vec::new();
  for (id, re) in res.into_iter().enumerate() {
    match parse(re.as_ref()) {
      Ok(re) => dfas.push(Dfa::from_nfa(&Nfa::from_re(&re), id as u32).minimize()),
      Err(err) => return Err((id, err)),
    }
  }
  let dfa = Dfa::merge(&dfas);
  let ec = ec_of_dfas(&[&dfa]);
  Ok((dfa, ec))
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a, T> {
  pub ty: T,
  pub piece: &'a [u8],
  pub line: u32,
  pub col: u32,
}
