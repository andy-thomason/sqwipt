use std::ops::Deref;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Punct(&'a str),
    Int(&'a str),
    Float(&'a str),
    Hex(&'a str),
    Keyword(&'a str),
    Ident(&'a str),
    Str(&'a str),
    Newline(&'a str),
    Begin(&'a str),
    End(&'a str),
    Eof(&'a str),
    UnknownToken(&'a str),
    UnterminatedString(&'a str),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Span<'a>(&'a str);

impl<'a> Deref for Span<'a> {
    type Target = &'a str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Lex<'a> {
    src: &'a str,
    indent: Vec<usize>,
    pos: usize,
    peek: Token<'a>,
}

impl<'a> Lex<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut s = Self {
            src,
            indent: vec![],
            pos: 0,
            peek: Token::UnknownToken(""),
        };
        s.peek = s.next();
        s
    }

    pub fn error(&mut self, span: Span<'a>, text: String) {
        eprintln!("error {text} at {span:?}");
    }

    pub fn peek(&self) -> &Token<'a> {
        &self.peek
    }

    pub fn advance(&mut self) -> Span<'a> {
        let span = self.span();
        self.peek = self.next();
        span
    }

    pub fn span(&self) -> Span<'a> {
        Span(match self.peek() {
            Token::Punct(span) |
            Token::Int(span) |
            Token::Float(span) |
            Token::Hex(span) |
            Token::Keyword(span) |
            Token::Ident(span) |
            Token::Str(span) |
            Token::Newline(span) |
            Token::Begin(span) |
            Token::End(span) |
            Token::Eof(span) |
            Token::UnknownToken(span) |
            Token::UnterminatedString(span) => span,
        })
    }

    pub fn next(&mut self) -> Token<'a> {
        let bytes = self.src.as_bytes();
        let mut pos = next_pos(bytes, self.pos, |b| b != b' ');
        let start = pos;
        if pos == bytes.len() {
            self.pos = pos;
            if self.indent.is_empty() {
                Token::Eof(&self.src[start..pos])
            } else {
                self.indent.pop();
                Token::End(&self.src[start..pos])
            }
        } else {
            self.pos = pos + 1;
            match bytes[pos] {
                b if b.is_ascii_digit() => {
                    if bytes[pos..].starts_with(b"0x") {
                        self.pos = next_pos(bytes, pos+2, |b| !b.is_ascii_hexdigit());
                        Token::Hex(&self.src[start..self.pos])
                    } else {
                        pos = next_pos(bytes, pos+1, |b| !b.is_ascii_digit());
                        let mut is_float = false;
                        if bytes.get(pos) == Some(&b'.') {
                            pos = next_pos(bytes, pos+1, |b| !b.is_ascii_digit());
                            is_float = true;
                        }
                        if bytes.get(pos).map(u8::to_ascii_lowercase) == Some(b'e') {
                            is_float = true;
                            if bytes.get(pos) == Some(&b'+') || bytes.get(pos) == Some(&b'-') {
                                pos += 1;
                            }
                            pos = next_pos(bytes, pos+1, |b| !b.is_ascii_digit());
                        }
                        self.pos = pos;
                        if is_float {
                            Token::Float(&self.src[start..pos])
                        } else {
                            Token::Int(&self.src[start..pos])
                        }
                    }
                }
                b if b.is_ascii_alphabetic() => {
                    pos = next_pos(bytes, pos+1, |b| !b.is_ascii_alphanumeric() && b != b'_');
                    self.pos = pos;
                    let span = &self.src[start..pos];
                    let is_keyword = match span {
                        "def" => true,
                        "if" => true,
                        "else" => true,
                        "for" => true,
                        "let" => true,
                        "mut" => true,
                        _ => false,
                    };
                    // eprintln!("k({span}, {is_keyword})");
                    if is_keyword {
                        Token::Keyword(span)
                    } else {
                        Token::Ident(span)
                    }
                }
                b'"'| b'\'' => {
                    let terminator = bytes[pos];
                    if let Some(pos) = bytes[pos+1..].windows(2).position(|s| s[0] != b'\\' && s[1] == terminator) {
                        self.pos = pos + 3;
                        Token::Str(&self.src[start..self.pos])
                    } else {
                        self.pos = bytes.len();
                        Token::UnterminatedString(&self.src[start..self.pos])
                    }
                }

                b'\n' => {
                    pos += 1;
                    let start = pos;
                    pos = next_pos(bytes, pos, |b| b != b' ');
                    let new_indent = pos - start;
                    let old_indent = self.indent.last().copied().unwrap_or_default();
                    eprintln!("{:?} old={old_indent} new={new_indent}", self.indent);
                    if pos == bytes.len() {
                        self.pos = pos;
                        Token::Newline(&self.src[pos..pos])
                    } else if bytes[pos] == b'\n' {
                        Token::Newline(&self.src[pos-1..pos])
                    } else {
                        self.pos = pos;
                        if new_indent > old_indent {
                            self.indent.push(new_indent);
                            Token::Begin(&self.src[start..start])
                        } else if new_indent < old_indent {
                            self.indent.pop();
                            Token::End(&self.src[start..start])
                        } else {
                            Token::Newline(&self.src[start..start])
                        }
                    }
                }

                _ => {
                    let bp = &bytes[pos..];
                    const PUNCT : &'static [&'static [u8]] = &[
                        b">>>", 
                        b"**", b"<<", b">>", b"+=", b"-=", b"/=",
                        b"!", b"+", b"-", b"*", b"/", b"=", b"[", b"]", b"(", b")", b":", b",", b";", b".",
                    ];
            
                    if let Some(p) = PUNCT.iter().find(|p| bp.starts_with(p)) {
                        self.pos = pos + p.len();
                        Token::Punct(&self.src[start..self.pos])
                    } else {
                        self.pos = pos + 1;
                        Token::UnknownToken(&self.src[start..self.pos])
                    }
                }
            }
        }
    }
}

fn next_pos<F : Fn(u8) -> bool>(bytes: &[u8], pos: usize, pred: F) -> usize {
    pos + bytes[pos..].iter().position(|&b| pred(b)).unwrap_or(bytes[pos..].len())
}
