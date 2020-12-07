pub mod re;
pub mod nfa;
pub mod dfa;
pub mod print;

pub use nfa::*;
pub use dfa::*;
use std::hash::BuildHasherDefault;
use ahash::AHasher;

// `BuildHasherDefault<AHasher>` will call `AHasher::default` to generate the hasher,
// which guarantees to have the same initial state, so the `HashMap/Set` will be somewhat deterministic
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;
pub type HashSet<K> = std::collections::HashSet<K, BuildHasherDefault<AHasher>>;

// return Err((idx, reason)): the `idx`th regex in `re` is invalid because of the syntax error described in`reason`
pub fn re2dfa<'a>(re: impl IntoIterator<Item=&'a [u8]>) -> Result<Dfa, (usize, String)> {
  let nfa = Nfa::from_re(re)?;
  let mut dfa = Dfa::from_nfa(&nfa);
  dfa.minimize();
  Ok(dfa)
}