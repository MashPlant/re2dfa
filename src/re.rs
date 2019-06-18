// theoretically Concat & Disjunction only need 2 children
// but use a Vec here can make future analysis faster

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Re {
  Ch(u8),
  Concat(Vec<Re>),
  Disjunction(Vec<Re>),
  Kleene(Box<Re>),
}

pub fn parse(s: &str) -> Result<Re, (&'static str, usize)> {
  #[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
  enum Opt {
    LeftParam,
    Disjunction,
    Concat,
  }
  let (mut opr, mut opt): (_, Vec<(Opt, usize)>) = (Vec::new(), Vec::new());
  let mut escape = false;

  macro_rules! handle_op {
    ($idx: expr, $var: ident) => {
      match (opr.pop(), opr.pop()) {
        (Some(r), Some(l)) => {
          // do a lot of extra job to reduce the depth of expression tree...
          opr.push(Re::$var(match (l, r) {
            (Re::$var(mut l), Re::$var(mut r)) => (l.append(&mut r), l).1,
            (Re::$var(mut l), r) => (l.push(r), l).1,
            (l, Re::$var(mut r)) => (r.push(l), r).1,
            (l, r) => vec![l, r],
          }))
        }
        _ => return Err(("binary operator misses operand", $idx))
      }
    };
  }

  for ((idx, byte), nxt) in s.bytes().enumerate().zip(s.bytes().skip(1).chain(b')'..=b')')) {
    macro_rules! add_concat {
      () => {
        // quite dirty... I don't know whether this is enough
        if nxt != b')' && nxt != b'*' && nxt != b'+' && nxt != b'|' {
          opt.push((Opt::Concat, idx));
        }
      };
    }

    if escape {
      escape = false;
      match byte {
        b'n' => opr.push(Re::Ch(b'\n')),
        b't' => opr.push(Re::Ch(b'\t')),
        b'\\' => opr.push(Re::Ch(b'\\')),
        b'd' => opr.push(Re::Disjunction((b'0'..=b'9').map(|it| Re::Ch(it)).collect())),
        b's' => opr.push(Re::Disjunction(("\t\n\r ".bytes()).map(|it| Re::Ch(it)).collect())),
        b => opr.push(Re::Ch(b)),
      }
      while let Some(op) = opt.last().map(|it| *it) {
        if op.0 < Opt::Concat { break; }
        opt.pop();
        handle_op!(op.1, Concat);
      }
      add_concat!();
    } else {
      match byte {
        b'\\' => escape = true,
        b'*' => {
          match opr.pop() {
            Some(l) => opr.push(Re::Kleene(Box::new(l))),
            None => return Err(("unary operator misses operand", idx))
          }
          add_concat!();
        }
        b'+' => {
          match opr.pop() {
            Some(l) => opr.push(Re::Concat(vec![l.clone(), Re::Kleene(Box::new(l))])),
            None => return Err(("unary operator misses operand", idx))
          }
          add_concat!();
        }
        b'|' => {
          while let Some(op) = opt.last().map(|it| *it) {
            if op.0 < Opt::Disjunction { break; }
            opt.pop();
            match op.0 {
              Opt::Disjunction => handle_op!(op.1, Disjunction),
              Opt::Concat => handle_op!(op.1, Concat),
              _ => unreachable!(),
            }
          }
          opt.push((Opt::Disjunction, idx));
        }
        b'(' => opt.push((Opt::LeftParam, idx)),
        b')' => {
          while let Some(op) = opt.pop() {
            match op.0 {
              Opt::Disjunction => handle_op!(op.1, Disjunction),
              Opt::Concat => handle_op!(op.1, Concat),
              Opt::LeftParam => break,
            }
          }
          add_concat!();
        }
        b'.' => {
          // accept all printable ascii char, quite brutal
          opr.push(Re::Disjunction((b' '..=b'~').map(|it| Re::Ch(it)).collect()));
          while let Some(op) = opt.last().map(|it| *it) {
            if op.0 < Opt::Concat { break; }
            opt.pop();
            handle_op!(op.1, Concat);
          }
          add_concat!();
        }
        b => {
          opr.push(Re::Ch(b));
          while let Some(op) = opt.last().map(|it| *it) {
            if op.0 < Opt::Concat { break; }
            opt.pop();
            handle_op!(op.1, Concat);
          }
          add_concat!();
        }
      }
    }
  }

  while let Some(op) = opt.pop() {
    match op.0 {
      Opt::Disjunction => handle_op!(op.1, Disjunction),
      Opt::Concat => handle_op!(op.1, Concat),
      Opt::LeftParam => return Err(("params mismatch", op.1)),
    }
  }
  Ok(opr.pop().unwrap())
}