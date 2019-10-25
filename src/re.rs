use nom::{branch::alt, bytes::complete::tag, character::complete::{char, one_of, none_of}, combinator::{map, cut}, error::{convert_error, ParseError, VerboseError}, multi::{separated_list, many1}, sequence::{preceded, terminated, tuple}, Err, IResult};
use std::{collections::HashSet, str};

// theoretically Concat & Disjunction only need 2 children
// but use a Vec here can make future analysis faster
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Re {
  Eps,
  Ch(u8),
  Concat(Vec<Re>),
  Disjunction(Vec<Re>),
  Kleene(Box<Re>),
}

use Re::*;
use nom::bytes::complete::take_while_m_n;
use nom::combinator::map_opt;

// our simple re doesn't support {n},^,$, but still them as meta chars
const META: &str = r"()[].|*+\{}^$?";

// it is called escaped_ascii instead of ascii_escaped, because it only accept escape chars
// '\' is considered as both a meta char and a normal ascii escape cha
fn escaped_ascii<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u8, E> {
  alt((
    map(tag(r#"\""#), |_| b'\"'),
    map(tag(r"\\"), |_| b'\\'),
    map(tag(r"\n"), |_| b'\n'),
    map(tag(r"\t"), |_| b'\t'),
    map(tag(r"\r"), |_| b'\r'),
    map(preceded(tag(r"\x"), take_while_m_n(2, 2, |ch: char| ch.is_digit(16))), |s| u8::from_str_radix(s, 16).unwrap()),
  ))(i)
}

fn atom<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Re, E> {
  alt((
    map(none_of(META), |ch| if ch.len_utf8() == 1 { Ch(ch as u8) } else { Concat(ch.encode_utf8(&mut [0; 4]).bytes().map(|ch| Ch(ch)).collect()) }),
    map(escaped_ascii, |ch| Ch(ch)),
    map(tag(r"\d"), |_| Disjunction((b'0'..=b'9').map(|ch| Ch(ch)).collect())),
    map(tag(r"\w"), |_| Disjunction((b'0'..=b'9').chain(b'a'..=b'z').chain(b'A'..=b'Z').chain(Some(b'_')).map(|ch| Ch(ch)).collect())),
    map(tag(r"\s"), |_| Disjunction("\n\t\r ".bytes().map(|ch| Ch(ch)).collect())),
    map(char('.'), |_| Disjunction((0..=255).map(|ch| Ch(ch)).collect())),
    preceded(char('\\'), map(cut(one_of(META)), |ch| Ch(ch as u8))),
    preceded(char('('), cut(terminated(re, char(')')))),
    range,
  ))(i)
}

fn atom_with_suffix<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Re, E> {
  alt((
    map(terminated(atom, char('*')), |a| Kleene(Box::new(a))),
    map(terminated(atom, char('+')), |a| Concat(vec![a.clone(), Kleene(Box::new(a))])),
    map(terminated(atom, char('?')), |a| Disjunction(vec![Eps, a])),
    atom,
  ))(i)
}

// meta characters are not escaped here, but other normal ascii escape chars and [] are
// multi-byte char is not supported in []
fn ascii_no_bracket<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u8, E> {
  alt((
    map(tag(r"\["), |_| b'['),
    map(tag(r"\]"), |_| b']'),
    escaped_ascii,
    map_opt(none_of(r"\[]"), |ch| if ch.len_utf8() == 1 { Some(ch as u8) } else { None }),
  ))(i)
}

fn range<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Re, E> {
  // Fn doesn't implement Clone, so can't store the result to a variable and use it twice
  macro_rules! ranges {
    () => {
      cut(map(many1(alt((
        map(tuple((ascii_no_bracket, char('-'), ascii_no_bracket)), |(l, _, u)| (l, u)),
        map(ascii_no_bracket, |x| (x, x)),
      ))), |rs| rs.iter().flat_map(|&(l, u)| l..=u).collect::<HashSet<_>>()))
    };
  }
  preceded(char('['), cut(terminated(alt((
    map(preceded(char('^'), ranges!()), |r| Disjunction((0..=255).filter(|x| !r.contains(x)).map(|ch| Ch(ch)).collect())),
    map(ranges!(), |r| Disjunction(r.into_iter().map(|ch| Ch(ch)).collect::<Vec<_>>())),
  )), char(']'))))(i)
}

fn re<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Re, E> {
  let (i, disjunction) = separated_list(char('|'), map(many1(atom_with_suffix), Concat))(i)?;
  Ok((i, match disjunction.len() { 0 => Eps, 1 => disjunction.into_iter().next().unwrap(), _ => Disjunction(disjunction) }))
}

pub fn parse(i: &str) -> Result<Re, String> {
  let result = re::<VerboseError<&str>>(i);
  match result {
    Ok(("", result)) => Ok(result),
    Ok((remain, _)) => Err(format!("remaining part cannot be parsed: {:?}", remain)),
    Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(convert_error(i, e)),
    // we don't use nom's stream mode, so won't have this error
    Err(Err::Incomplete(_)) => unreachable!()
  }
}