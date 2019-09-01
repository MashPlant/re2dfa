use nom::{
  branch::alt,
  bytes::complete::tag,
  character::complete::{char, one_of},
  combinator::{map, cut},
  error::{convert_error, ErrorKind, ParseError, VerboseError},
  multi::separated_list,
  sequence::{preceded, terminated},
  bytes::complete::take_while_m_n,
  character::complete::anychar,
  multi::many1,
  sequence::tuple,
  Err, IResult,
};
use std::collections::HashSet;
use std::str;

// theoretically Concat & Disjunction only need 2 children
// but use a Vec here can make future analysis faster

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Re {
  Ch(u8),
  Concat(Vec<Re>),
  Disjunction(Vec<Re>),
  Kleene(Box<Re>),
}

// out simple re doesn't support {n},^,$,?, but still them as meta chars
const META: &'static str = r"()[].|*+\{}^$?";

fn parse_atom<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Re, E> {
  alt((
    map(take_while_m_n(1, 1, |ch| !META.contains(ch)), |s: &'a str| Re::Ch(s.as_bytes()[0])),
    map(tag(r"\n"), |_| Re::Ch(b'\n')),
    map(tag(r"\r"), |_| Re::Ch(b'\r')),
    map(tag(r"\t"), |_| Re::Ch(b'\t')),
    map(tag(r"\{"), |_| Re::Ch(b'{')), 
    map(tag(r"\}"), |_| Re::Ch(b'}')),
    map(tag(r"\^"), |_| Re::Ch(b'^')),
    map(tag(r"\$"), |_| Re::Ch(b'$')),
    map(tag(r"\?"), |_| Re::Ch(b'?')),
    map(tag(r"\d"), |_| Re::Disjunction((b'0'..=b'9').map(|it| Re::Ch(it)).collect())),
    map(tag(r"\s"), |_| Re::Disjunction("\t\n\r ".bytes().map(|it| Re::Ch(it)).collect())),
    map(tag(r"."), |_| Re::Disjunction((0..=255).map(|it| Re::Ch(it)).collect())),
    preceded(tag(r"\"), map(cut(one_of(META)), |ch| Re::Ch(ch as u8))),
    preceded(char('('), cut(terminated(parse_re, char(')')))),
    parse_range,
  ))(i)
}

fn parse_atom_or_kleene<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Re, E> {
  alt((
    map(terminated(parse_atom, tag("*")), |a| Re::Kleene(Box::new(a))),
    map(terminated(parse_atom, tag("+")), |a| Re::Concat(vec![a.clone(), Re::Kleene(Box::new(a))])),
    parse_atom,
  ))(i)
}

// I thought nom should have provided such a function...
// can escaped & escaped_transform of help here?
fn ascii<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u8, E> {
  alt((
    map(tag(r"\'"), |_| b'\''),
    map(tag(r#"\""#), |_| b'\"'),
    map(tag(r"\\"), |_| b'\\'),
    map(tag(r"\n"), |_| b'\n'),
    map(tag(r"\r"), |_| b'\r'),
    map(tag(r"\t"), |_| b'\t'),
    map(anychar, |ch| ch as u8),
  ))(i)
}

// oh pls someone teach me how to use nom...
// will not be of help here?
fn ascii_bracket<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, u8, E> {
  match one_of::<&'a str, &'a str, E>("[]")(i) {
    Ok(_) => Err(Err::Error(E::from_error_kind(i, ErrorKind::Not))),
    _ => alt((
      map(tag(r"\["), |_| b'['),
      map(tag(r"\]"), |_| b']'),
      ascii
    ))(i),
  }
}

fn parse_range<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Re, E> {
  // why this is not Copy???
  macro_rules! ranges {
    () => {
      map(many1(alt((
        map(tuple((ascii_bracket, tag("-"), ascii_bracket)), |(l, _, u)| (l, u)),
        map(ascii_bracket, |x| (x, x)),
      ))), |ranges| {
        let mut range = HashSet::new();
        for (l, u) in ranges {
          for i in l..=u {
            range.insert(i);
          }
        }
        range
      })
    };
  }
  preceded(tag("["), cut(terminated(alt((
    map(preceded(tag("^"), cut(ranges!())), |range| {
      Re::Disjunction((0..=255).filter(|x| !range.contains(x)).map(|it| Re::Ch(it)).collect())
    }),
    map(ranges!(), |range| {
      let mut range = range.into_iter().collect::<Vec<_>>();
      range.sort_unstable();
      Re::Disjunction(range.into_iter().map(|ch| Re::Ch(ch)).collect::<Vec<_>>())
    }),
  )), tag("]"))))(i)
}

fn parse_concat<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Re>, E> {
  many1(parse_atom_or_kleene)(i)
}

fn parse_disjunction<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<Re>, E> {
  // eliminate left recursion???
  let x = terminated(parse_concat, tag("|"))(i)?;
  let mut xs = separated_list(tag("|"), parse_re)(x.0)?;
  xs.1.push(Re::Concat(x.1));
  Ok((xs.0, xs.1))
}

fn parse_re<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Re, E> {
  alt((
    map(parse_disjunction, Re::Disjunction),
    map(parse_concat, Re::Concat),
  ))(i)
}

pub fn parse(i: &str) -> Result<Re, String> {
  let result = parse_re::<VerboseError<&str>>(i);
  match result {
    Ok(("", result)) => Ok(result),
    Ok((remain, _)) => Err(format!("The remaining part cannot be parser: `{}`.", remain)),
    Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(convert_error(i, e)),
    // what is it???
    Err(Err::Incomplete(_)) => unreachable!()
  }
}
