use nom::{branch::alt, bytes::complete::tag, combinator::{map, cut}, multi::{separated_list0, many1}, sequence::{preceded, terminated, tuple}, Err, error::{ErrorKind, Error}, IResult};
use Re::*;

#[derive(Debug, Clone)]
pub enum Re {
  Eps,
  Ch(u8),
  // theoretically Concat & Disjunction only need 2 children, but using Vec here can make future analysis faster
  Concat(Box<[Re]>),
  Disjunction(Box<[Re]>),
  // DisjunctionCh(bitset) == Disjunction([(ones in bitset).map(Ch)])
  // this is a more efficient way to represent [] in regex
  DisjunctionCh(Box<[u32; 8]>),
  Kleene(Box<Re>),
}

// our simple implementation doesn't support {n},^,$, but still regard them as meta chars
const META: &[u8] = br"()[].|*+\{}^$?";

macro_rules! err {
  ($i: expr, $code: ident) => { Err(Err::Error(Error::new($i, ErrorKind::$code))) };
}

fn escaped_ascii<'a>(i: &'a [u8]) -> IResult<&'a [u8], u8> {
  alt((
    map(tag(br#"\""#), |_| b'\"'),
    map(tag(br"\\"), |_| b'\\'),
    map(tag(br"\n"), |_| b'\n'),
    map(tag(br"\t"), |_| b'\t'),
    map(tag(br"\r"), |_| b'\r'),
    preceded(tag(br"\x"), |i: &'a [u8]| {
      if let [hi, lo, ref i @ ..] = i {
        let hex = |x| match x {
          b'0'..=b'9' => Some(x - b'0'), b'a'..=b'f' => Some(x - b'a' + 10), b'A'..=b'F' => Some(x - b'A' + 10), _ => None
        };
        if let (Some(hi), Some(lo)) = (hex(*hi), hex(*lo)) { return Ok((i, hi * 16 + lo)); }
      }
      err!(i, HexDigit)
    }),
  ))(i)
}

#[inline(always)]
fn byte(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], ()> {
  move |i| { match i { [x, ref i @ ..] if *x == b => Ok((i, ())), _ => err!(i, Char) } }
}

#[inline(always)]
fn none_of(s: &'static [u8]) -> impl Fn(&[u8]) -> IResult<&[u8], u8> {
  move |i| { match i { [x, ref i @ ..] if !s.contains(x) => Ok((i, *x)), _ => err!(i, NoneOf) } }
}

#[inline(always)]
fn one_of(s: &'static [u8]) -> impl Fn(&[u8]) -> IResult<&[u8], u8> {
  move |i| { match i { [x, ref i @ ..] if s.contains(x) => Ok((i, *x)), _ => err!(i, OneOf) } }
}

fn atom(i: &[u8]) -> IResult<&[u8], Re> {
  alt((
    map(none_of(META), Ch),
    map(escaped_ascii, Ch),
    // equivalent to `Disjunction((b'0'..=b'9').map(Ch).collect())`
    map(tag(br"\d"), |_| DisjunctionCh([0, 0b11111111110000000000000000, 0, 0, 0, 0, 0, 0].into())),
    // equivalent to `Disjunction((b'0'..=b'9').chain(b'..=b'z').chain(b'..=b'Z').chain(Some(b'_')).map(Ch).collect())`
    map(tag(br"\w"), |_| DisjunctionCh([0, 0b11111111110000000000000000, 0b10000111111111111111111111111110, 0b111111111111111111111111110, 0, 0, 0, 0].into())),
    // equivalent to `Disjunction("\n\t\r ".bytes().map(Ch).collect())`
    map(tag(br"\s"), |_| DisjunctionCh([0b10011000000000, 0b1, 0, 0, 0, 0, 0, 0].into())),
    // equivalent to `Disjunction((0..=255).map(Ch).collect())`
    map(byte(b'.'), |_| DisjunctionCh([!0; 8].into())),
    preceded(byte(b'\\'), map(cut(one_of(META)), Ch)),
    preceded(byte(b'('), cut(terminated(re, byte(b')')))),
    range,
  ))(i)
}

fn atom_with_suffix(i: &[u8]) -> IResult<&[u8], Re> {
  let (i, a) = atom(i)?;
  Ok(match i {
    [b'*', ref i @ ..] => (i, Kleene(Box::new(a))),
    [b'+', ref i @ ..] => (i, Concat([a.clone(), Kleene(Box::new(a))].into())),
    [b'?', ref i @ ..] => (i, Disjunction([Eps, a].into())),
    _ => (i, a),
  })
}

// meta characters are not escaped here, but other normal ascii escape chars and [] are
// multi-byte char is not supported in []
fn ascii_no_bracket(i: &[u8]) -> IResult<&[u8], u8> {
  alt((
    map(tag(br"\["), |_| b'['),
    map(tag(br"\]"), |_| b']'),
    escaped_ascii,
    none_of(br"\[]"),
  ))(i)
}

fn range<'a>(i: &'a [u8]) -> IResult<&'a [u8], Re> {
  // basically copied from `nom::multi::many1`, but avoid allocating a Vec as the result
  preceded(byte(b'['), cut(terminated(|i: &'a [u8]| {
    let (mut i, inv) = match i { [b'^', ref i @ ..] => (i, true), _ => (i, false) };
    let mut set = [0; 8];
    loop {
      match alt((
        map(tuple((ascii_no_bracket, byte(b'-'), ascii_no_bracket)), |(l, _, u)| (l, u)),
        map(ascii_no_bracket, |x| (x, x)),
      ))(i) {
        Err(Err::Error(_)) => break,
        Err(e) => return Err(e),
        Ok((i1, (l, u))) => {
          i = i1;
          for x in l..=u { bitset::bs(&mut set).set(x as usize); }
        }
      }
    }
    if inv { bitset::bs(&mut set).inv(); }
    Ok((i, DisjunctionCh(set.into())))
  }, byte(b']'))))(i)
}

fn re(i: &[u8]) -> IResult<&[u8], Re> {
  // currently for a Vec with len == 1, the range check in `.remove(0)` can be optimized out
  // but the check in `.into_iter().next().unwrap()` cannot, so I choose the former
  let (i, mut d) = separated_list0(byte(b'|'), map(many1(atom_with_suffix), |mut c|
    match c.len() { 1 => c.remove(0), _ => Concat(c.into()) }))(i)?;
  Ok((i, match d.len() {
    0 => Eps, 1 => d.remove(0), _ => {
      let mut set = [0; 8];
      // if all possibilities are `Ch` or `DisjunctionCh`, this `Disjunction` can be simplified
      if d.iter().all(|x| match x {
        &Ch(ch) => (bitset::bs(&mut set).set(ch as usize), true).1,
        DisjunctionCh(s) => (bitset::bs(&mut set).or(s.as_ref()), true).1,
        _ => false,
      }) { DisjunctionCh(set.into()) } else { Disjunction(d.into()) }
    }
  }))
}

pub fn parse(i: &[u8]) -> Result<Re, String> {
  match re(i) {
    Ok((b"", result)) => Ok(result),
    Ok((remain, _)) => Err(format!("remaining part cannot be parsed: {:?}", remain)),
    Err(e) => Err(format!("{}", e)),
  }
}