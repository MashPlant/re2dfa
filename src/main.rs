#[macro_use]
extern crate smallvec;
extern crate nom;
extern crate bitset;
extern crate print;

use crate::re::Re;
use crate::dfa::Dfa;
use crate::nfa::Nfa;

pub mod re;
pub mod nfa;
pub mod dfa;

fn re2dfa(re: &Re, id: u32) -> Dfa {
  Dfa::from_nfa(&Nfa::from_re(re), id).minimize()
}

fn main() {
  let dfa = [
    &re::parse("void").unwrap(),
    &re::parse("int").unwrap(),
    &re::parse("bool").unwrap(),
    &re::parse("string").unwrap(),
    &re::parse("new").unwrap(),
    &re::parse("null").unwrap(),
    &re::parse("true").unwrap(),
    &re::parse("false").unwrap(),
    &re::parse("class").unwrap(),
    &re::parse("extends").unwrap(),
    &re::parse("this").unwrap(),
    &re::parse("while").unwrap(),
    &re::parse("foreach").unwrap(),
    &re::parse("for").unwrap(),
    &re::parse("if").unwrap(),
    &re::parse("else").unwrap(),
    &re::parse("return").unwrap(),
    &re::parse("break").unwrap(),
    &re::parse("Print").unwrap(),
    &re::parse("ReadInteger").unwrap(),
    &re::parse("ReadLine").unwrap(),
    &re::parse("static").unwrap(),
    &re::parse("instanceof").unwrap(),
    &re::parse("scopy").unwrap(),
    &re::parse("sealed").unwrap(),
    &re::parse("var").unwrap(),
    &re::parse("default").unwrap(),
    &re::parse("in").unwrap(),
    &re::parse(r"\|\|\|").unwrap(),
    &re::parse("<=").unwrap(),
    &re::parse(">=").unwrap(),
    &re::parse("==").unwrap(),
    &re::parse("!=").unwrap(),
    &re::parse("&&").unwrap(),
    &re::parse(r"\|\|").unwrap(),
    &re::parse("%%").unwrap(),
    &re::parse(r"\+\+").unwrap(),
    &re::parse("--").unwrap(),
    &re::parse("<<").unwrap(),
    &re::parse(">>").unwrap(),
    &re::parse(r"\+").unwrap(),
    &re::parse("-").unwrap(),
    &re::parse(r"\*").unwrap(),
    &re::parse("/").unwrap(),
    &re::parse("%").unwrap(),
    &re::parse("&").unwrap(),
    &re::parse(r"\|").unwrap(),
    &re::parse("^").unwrap(),
    &re::parse("=").unwrap(),
    &re::parse("<").unwrap(),
    &re::parse(">").unwrap(),
    &re::parse(r"\.").unwrap(),
    &re::parse(",").unwrap(),
    &re::parse(";").unwrap(),
    &re::parse("!").unwrap(),
    &re::parse(r"\(").unwrap(),
    &re::parse(r"\)").unwrap(),
    &re::parse(r"\[").unwrap(),
    &re::parse(r"\]").unwrap(),
    &re::parse("{").unwrap(),
    &re::parse("}").unwrap(),
    &re::parse(":").unwrap(),
    &re::parse(r#"""#).unwrap(),
    &re::parse(r"//[^\n]*").unwrap(),
    &re::parse(r"\d+").unwrap(),
    &re::parse(r"\s+").unwrap(),
    &re::parse("[a-zA-Z][_a-zA-Z0-9]*").unwrap(),
  ].iter()
    .enumerate()
    .map(|(idx, re)| re2dfa(re, idx as u32))
    .collect::<Vec<_>>();
  let merged = dfa::Dfa::merge(&dfa).minimize();
  println!("{}", merged.nodes.len());

  use std::fs::File;
  use std::io::prelude::*;
  let mut f = File::create("merged.dot").unwrap();
  f.write(merged.print_dot().as_bytes()).unwrap();
}
