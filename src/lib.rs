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

// return value:
// Ok((dfa, ec)): ec is short for equivalent char, any u8 can be the index in the array
// if two u8 has the same value in the array, they can be treated in the same way in `dfa`
// Err(reason): one of the regex in `re` isn't a valid regex expression, `reason` describes more information about the syntax error
pub fn re2dfa<I: IntoIterator<Item=S>, S: AsRef<str>>(re: I) -> Result<(Dfa, [u8; 256]), (usize, String)> {
  let mut dfas = Vec::new();
  for (id, re) in re.into_iter().enumerate() {
    match parse(re.as_ref()) {
      Ok(re) => dfas.push(Dfa::from_nfa(&Nfa::from_re(&re), id as u32).minimize()),
      Err(err) => return Err((id, err)),
    }
  }
  let dfa = Dfa::merge(&dfas);
  let ec = ec_of_dfas(&[&dfa]);
  Ok((dfa, ec))
}