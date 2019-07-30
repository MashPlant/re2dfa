pub struct {{T}}Lexer<'a> {
  pub string: &'a [u8],
  pub cur_line: u32,
  pub cur_col: u32,
}

impl<'a> {{T}}Lexer<'a> {
  pub fn new(string: &'a [u8]) -> Self {
    Self {
      string,
      cur_line: 1,
      cur_col: 1,
    }
  }

  pub fn next(&mut self) -> Option<re2dfa::Token<'a, {{T}}>> {
    #[cfg(not(feature = "unsafe_parser"))]
    macro_rules! index {
      ($arr: expr, $idx: expr) => { $arr[$idx as usize] };
    }
    #[cfg(feature = "unsafe_parser")]
    macro_rules! index {
      ($arr: expr, $idx: expr) => { unsafe { *$arr.get_unchecked($idx as usize) } };
    }

    use {{T}}::*;
    static ACC: [{{T}}; {{DFA_SIZE}}] = [{{ACC}}];
    static EC: [u8; 256] = [{{EC}}];
    static EDGE: [[{{U_DFA_SIZE}}; {{EC_SIZE}}]; {{DFA_SIZE}}] = [{{DFA_EDGE}}];
    loop {
      if self.string.is_empty() {
        return Some(re2dfa::Token { ty: {{EOF}}, piece: "".as_bytes(), line: self.cur_line, col: self.cur_col });
      }
      let (mut line, mut col) = (self.cur_line, self.cur_col);
      let mut last_acc = {{EOF}}; // this is arbitrary, just a value that cannot be returned by user defined function
      let mut state = 0;
      let mut i = 0;
      while i < self.string.len() {
        let ch = index!(self.string, i);
        let ec = index!(EC, ch);
        let nxt = index!(index!(EDGE, state), ec);
        let acc = index!(ACC, nxt);
        last_acc = if acc != {{EOF}} { acc } else { last_acc };
        state = nxt;
        if nxt == 0 {
          if last_acc == {{EOF}} {
            return None;
          } else {
            let piece = &self.string[..i];
            self.string = &self.string[i..];
            {{HANDLE_EPS1}}
          }
        } else { // continue, eat this char
          if ch == b'\n' {
            self.cur_line += 1;
            self.cur_col = 1;
          } else {
            self.cur_col += 1;
          }
          i += 1;
        }
      }
      // end of file
      if last_acc == {{EOF}} { // completely dead
        return None;
      } else {
        let piece = & self.string[..i];
        self.string = & self.string[i..];
        {{HANDLE_EPS2}}
      }
    }
  }
}