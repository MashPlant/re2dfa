pub mod re;
pub mod nfa;
pub mod dfa;
pub mod print;

pub use re::{*, Re::*};
pub use nfa::*;
pub use dfa::*;
pub use print::*;

use tools::{*, fmt::*};

// return Err((idx, reason)): `re[idx]` is invalid because of the syntax error described in`reason`
pub fn re2dfa<'a>(re: impl IntoIterator<Item=&'a [u8]>) -> Result<Dfa, (usize, String)> {
  let nfa = Nfa::from_re(re)?;
  let mut dfa = Dfa::from_nfa(&nfa);
  dfa.minimize();
  Ok(dfa)
}