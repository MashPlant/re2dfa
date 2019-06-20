#[macro_use]
extern crate smallvec;
extern crate nom;
extern crate bitset;
extern crate print;

pub mod compress;
pub mod re;
pub mod nfa;
pub mod dfa;

pub fn re2dfa<I: IntoIterator<Item=S>, S: AsRef<str>>(res: I) -> Result<(dfa::Dfa, [u8; 128]), String> {
  let mut dfas = Vec::new();
  for (id, re) in res.into_iter().enumerate() {
    dfas.push(dfa::Dfa::from_nfa(&nfa::Nfa::from_re(&re::parse(re.as_ref())?), id as u32));
  }
  let dfa = dfa::Dfa::merge(&dfas);
  let ec = compress::ec_of_dfas(&[&dfa]);
  Ok((dfa, ec))
}