pub struct IndentPrinter {
    indent: String,
    content: String,
}

impl IndentPrinter {
    const INDENT: u32 = 2;
    const INDENT_STR: &'static str = "  ";

    pub fn new() -> Self {
        Self { indent: String::new(), content: String::new() }
    }

    pub fn inc(&mut self) -> &mut Self {
        self.indent += IndentPrinter::INDENT_STR;
        self
    }

    pub fn dec(&mut self) -> &mut Self {
        for _ in 0..IndentPrinter::INDENT {
            self.indent.pop().unwrap();
        }
        self
    }

    pub fn ln(&mut self, s: impl AsRef<str>) -> &mut Self {
        self.content += self.indent.as_ref();
        self.content += s.as_ref();
        self.content.push('\n');
        self
    }

    pub fn lns(&mut self, s: impl AsRef<str>) -> &mut Self {
        for s in s.as_ref().split('\n') {
            self.ln(s);
        }
        self
    }

    pub fn finish(self) -> String {
        self.content
    }
}

// made slight difference to char::escape_default
// modify the printing form of \t \n \r ' ' , - to make them look better in a .dot file
pub fn pretty_ch_display(ch: u8) -> &'static str {
    match ch {
        b'\t' => r"\\t",
        b'\n' => r"\\n",
        b'\r' => r"\\r",
        b' ' => "' '",
        b'!' => "!",
        b'\"' => "\"",
        b'#' => "#",
        b'$' => "$",
        b'%' => "%",
        b'&' => "&",
        b'\'' => "\'",
        b'(' => "(",
        b')' => ")",
        b'*' => "*",
        b'+' => "+",
        b',' => "','",
        b'-' => "'-'",
        b'.' => ".",
        b'/' => "/",
        b'0' => "0",
        b'1' => "1",
        b'2' => "2",
        b'3' => "3",
        b'4' => "4",
        b'5' => "5",
        b'6' => "6",
        b'7' => "7",
        b'8' => "8",
        b'9' => "9",
        b':' => ":",
        b';' => ";",
        b'<' => "<",
        b'=' => "=",
        b'>' => ">",
        b'?' => "?",
        b'@' => "@",
        b'A' => "A",
        b'B' => "B",
        b'C' => "C",
        b'D' => "D",
        b'E' => "E",
        b'F' => "F",
        b'G' => "G",
        b'H' => "H",
        b'I' => "I",
        b'J' => "J",
        b'K' => "K",
        b'L' => "L",
        b'M' => "M",
        b'N' => "N",
        b'O' => "O",
        b'P' => "P",
        b'Q' => "Q",
        b'R' => "R",
        b'S' => "S",
        b'T' => "T",
        b'U' => "U",
        b'V' => "V",
        b'W' => "W",
        b'X' => "X",
        b'Y' => "Y",
        b'Z' => "Z",
        b'[' => "[",
        b'\\' => "\\",
        b']' => "]",
        b'^' => "^",
        b'_' => "_",
        b'`' => "`",
        b'a' => "a",
        b'b' => "b",
        b'c' => "c",
        b'd' => "d",
        b'e' => "e",
        b'f' => "f",
        b'g' => "g",
        b'h' => "h",
        b'i' => "i",
        b'j' => "j",
        b'k' => "k",
        b'l' => "l",
        b'm' => "m",
        b'n' => "n",
        b'o' => "o",
        b'p' => "p",
        b'q' => "q",
        b'r' => "r",
        b's' => "s",
        b't' => "t",
        b'u' => "u",
        b'v' => "v",
        b'w' => "w",
        b'x' => "x",
        b'y' => "y",
        b'z' => "z",
        b'{' => "{",
        b'|' => "|",
        b'}' => "}",
        b'~' => "~",
        _ => panic!("not covered"),
    }
}

// chs should be sorted
pub fn pretty_chs_display(chs: &[u8]) -> String {
    let mut text = String::new();
    {
        let mut i = 0;
        while i < chs.len() {
            let mut j = i;
            while j + 1 < chs.len() && chs[j + 1] == chs[j] + 1 { j += 1; }
            if j <= i + 1 {
                for i in i..=j {
                    text += &format!("{},", pretty_ch_display(chs[i]));
                }
            } else {
                text += &format!("{}-{},", pretty_ch_display(chs[i]), pretty_ch_display(chs[j]));
            }
            i = j + 1;
        }
    }
    text.pop();
    text
}