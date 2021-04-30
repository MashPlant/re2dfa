use re2dfa::*;
use clap::{App, Arg};
use std::{fs::File, io::{Result, Write, BufWriter}, fmt::Display};

fn write(path: &str, s: impl Display) -> Result<()> { write!(BufWriter::new(File::create(path)?), "{}", s) }

fn main() -> Result<()> {
  let m = App::new("show_fa")
    .arg(Arg::with_name("input").required(true))
    .arg(Arg::with_name("nfa").long("nfa").takes_value(true))
    .arg(Arg::with_name("raw_dfa").long("raw_dfa").takes_value(true).help("show the dfa directly converted from nfa"))
    .arg(Arg::with_name("dfa").long("dfa").takes_value(true).help("show the minimized dfa"))
    .get_matches();
  let input = m.value_of("input").unwrap();
  let re = re::parse(input.as_bytes()).expect("invalid regex");
  let nfa = Nfa::from_re1(&[re]);
  if let Some(path) = m.value_of("nfa") { write(path, nfa.print_dot())?; }
  let mut dfa = Dfa::from_nfa(&nfa);
  if let Some(path) = m.value_of("raw_dfa") { write(path, dfa.print_dot())?; }
  dfa.minimize();
  if let Some(path) = m.value_of("dfa") { write(path, dfa.print_dot())?; }
  Ok(())
}