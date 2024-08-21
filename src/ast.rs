use crate::lex::{Lex, Token, Span};

#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a> {
    begin: Span<'a>,
    items: Vec<Expr<'a>>,
    end: Span<'a>,
}

impl<'a> core::fmt::Display for Closure<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Closure { open, formal_args, close, body } = self;
        write!(f, "{open}")?;
        for (arg, sep) in formal_args {
            write!(f, "{arg}")?;
            if let Some(sep) = sep {
                write!(f, "{sep} ")?;
            }
        }
        write!(f, "{close} {body}")
    }
}

impl<'a> core::fmt::Display for Block<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "scope")
    }
}

impl<'a> core::fmt::Display for FormalArg<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "arg")
    }
}

impl<'a> core::fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expr")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FormalArg<'a> {
    Name(Span<'a>),
    NameWithDefault(Span<'a>, Span<'a>, Expr<'a>),
    Bad(Span<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Closure<'a> {
    open: Span<'a>,
    formal_args: Vec<(FormalArg<'a>, Option<Span<'a>>)>,
    close: Span<'a>,
    body: Expr<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    Ident(Span<'a>),
    Int(Span<'a>),
    Float(Span<'a>),
    Hex(Span<'a>),
    Str(Span<'a>),
    Closure(Box<Closure<'a>>),
    Block(Block<'a>),
    Array(Span<'a>, Vec<Expr<'a>>, Span<'a>),
    Binary(Box<Expr<'a>>, Span<'a>, Box<Expr<'a>>),
    Unary(Span<'a>, Box<Expr<'a>>),
    Paren(Span<'a>, Box<Expr<'a>>, Span<'a>),
    Tuple(Span<'a>, Vec<(Expr<'a>, Option<Span<'a>>)>, Span<'a>),
    Call(Box<Expr<'a>>, Span<'a>, Vec<(Expr<'a>, Option<Span<'a>>)>, Span<'a>),
    Index(Box<Expr<'a>>, Span<'a>, Box<Expr<'a>>, Span<'a>),
    Dot(Box<Expr<'a>>, Span<'a>, Box<Expr<'a>>),
    Bad(Span<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    span: &'a str,
    reason: &'static str,
}

pub trait Parse<'l, 'a> : Sized {
    // Return either Some(item) or None if it cannot be one from
    // the first token.
    //
    // If there is a subsequent error, report it but still return
    // The type.

    fn parse(lex: &'l mut Lex<'a>) -> Option<Self>;
}

impl<'l, 'a> Parse<'l, 'a> for Expr<'a> {
    fn parse(lex: &'l mut Lex<'a>) -> Option<Self> {
        // println!("Expr::parse {:?}", lex.peek());
        match lex.peek() {
            Token::Begin(_) |
            Token::Punct("+") |
            Token::Punct("-") |
            Token::Punct("!") |
            Token::Punct("|") |
            Token::Punct("(") |
            Token::Int(_) |
            Token::Float(_) |
            Token::Hex(_) |
            Token::Ident(_) |
            Token::Str(_) => {
                Some(parse_binop(lex, 0))
            }
            _ => None
        }
    }
}

/// |x, y=5, z: u64 = 3| x + y + z
fn parse_closure<'a>(lex: &mut Lex<'a>) -> Expr<'a> {
    macro_rules! expect {
        ($token : pat) => {
            {
                if !matches!(lex.peek(), $token) {
                    return Expr::Bad(lex.advance());
                }
                lex.advance()
            }
        };
    }

    let open = expect!(Token::Punct("|"));
    let formal_args = parse_formal_args(lex);
    let close = expect!(Token::Punct("|"));
    if let Some(body) = Expr::parse(lex) {
        let function = Closure {
            open,
            formal_args,
            close,
            body,
        };
        Expr::Closure(Box::new(function))
    } else {
        lex.error(lex.span(), format!("expected expression after |..|"));
        Expr::Bad(lex.advance())
    }
}

fn parse_block<'a>(lex: &mut Lex<'a>) -> Expr<'a> {
    assert!(matches!(lex.peek(), Token::Begin(_)));
    let begin = lex.advance();
    let mut items = vec![];
    while !matches!(lex.peek(), Token::End(_)) && !matches!(lex.peek(), Token::Eof(_)) {
        if let Some(item) = Expr::parse(lex) {
            items.push(item);
        } else {
            lex.error(lex.span(), "expected expression".into());
            items.push(Expr::Bad(lex.advance()));
        }
        if matches!(lex.peek(), Token::Newline(_)) || matches!(lex.peek(), Token::Punct(";")) {
            lex.advance();
        } else if matches!(lex.peek(), Token::End(_)) || matches!(lex.peek(), Token::Eof(_)) {
            break;
        } else {
            lex.error(lex.span(), format!("expected newline or ; got {:?}", lex.peek()));
            items.push(Expr::Bad(lex.advance()));
        }
    }
    if matches!(lex.peek(), Token::End(_)) {
        let end = lex.advance();
        Expr::Block(Block {
            begin,
            items,
            end,
        })
    } else {
        let bad = lex.advance();
        lex.error(bad.clone(), format!("expected closing scope."));
        Expr::Bad(bad)
    }
}

/// A binop is a series of atoms joined by binary operators +, -, * etc.
fn parse_binop<'a>(lex: &mut Lex<'a>, min_precidence: usize) -> Expr<'a> {
    // println!("parse_binop {:?}", lex.peek());
    let mut lhs = parse_atom(lex);
    loop {
        let precidence = match lex.peek() {
            Token::Punct("+") => 30,
            Token::Punct("-") => 40,
            Token::Punct("*") => 40,
            Token::Punct("/") => 40,
            Token::Punct("%") => 40,
            Token::Punct("**") => 50,
            _ => break lhs,
        };

        if precidence < min_precidence {
            break lhs;
        } else {
            let span = lex.advance();
            let rhs = parse_binop(lex, precidence+1);
            lhs = Expr::Binary(Box::new(lhs), span, Box::new(rhs));
        }
    }
}

// An atom is the bread in the binop sandwich.
// 1, fred, "xyz", +1, fred[2], fred(1, 2, 3), [1, 2, 3], (1, 2, 3), (1+2)
// fred(1)(2) fred[1](2)
fn parse_atom<'a>(lex: &mut Lex<'a>) -> Expr<'a> {
    println!("parse_atom {:?}", lex.peek());
    let mut prefix = match lex.peek() {
        Token::Punct("|") => {
            parse_closure(lex)
        }
        Token::Begin(_) => {
            parse_block(lex)
        }
        Token::Punct("!") |
        Token::Punct("+") |
        Token::Punct("-") => {
            Expr::Unary(lex.advance(), Box::new(parse_atom(lex)))
        }
        Token::Punct("(") => {
            let lparen = lex.advance();
            let mut args = parse_args(lex);
            if matches!(args.as_slice(), &[(_, None)]) {
                let expr = args.pop().unwrap().0;
                Expr::Paren(lparen, Box::new(expr), parse_close(lex, ")"))
            } else {
                Expr::Tuple(lparen, args, parse_close(lex, ")"))
            }
        }
        Token::Ident(_) => {
            Expr::Ident(lex.advance())
        }
        Token::Int(_) => {
            Expr::Int(lex.advance())
        }
        Token::Float(_) => {
            Expr::Float(lex.advance())
        }
        Token::Hex(_) => {
            Expr::Hex(lex.advance())
        }
        Token::Str(_) => {
            Expr::Str(lex.advance())
        }
        _ => {
            let span = lex.advance();
            lex.error(span.clone(), format!("parse_atom"));
            return Expr::Bad(span)
        }
    };
    loop {
        prefix = match lex.peek() {
            Token::Punct("[") => {
                let lspan = lex.advance();
                let expr = parse_binop(lex, usize::MAX);
                let rspan = parse_close(lex, "]");
                Expr::Index(Box::new(prefix), lspan, Box::new(expr), rspan)
            }
            Token::Punct("(") => {
                let lspan = lex.advance();
                let args = parse_args(lex);
                let rspan = parse_close(lex, ")");
                Expr::Call(Box::new(prefix), lspan, args, rspan)
            }
            Token::Punct(".") => {
                let lspan = lex.advance();
                let rhs = parse_atom(lex);
                Expr::Dot(Box::new(prefix), lspan, Box::new(rhs))
            }
            _ => return prefix
        }
    }
}

// Parse a list of comma
fn parse_args<'a>(lex: &mut Lex<'a>) -> Vec<(Expr<'a>, Option<Span<'a>>)> {
    let mut args = vec![];
    while lex.peek() != &Token::Punct(")") {
        let expr = Expr::parse(lex)
            .unwrap_or_else(|| Expr::Bad(lex.advance()));
        if lex.peek() == &Token::Punct(",") {
            args.push((expr, Some(lex.advance())));
        } else {
            args.push((expr, None));
            break;
        }
    }
    args
}

// Parse formal args of a function.
fn parse_formal_args<'a>(lex: &mut Lex<'a>) -> Vec<(FormalArg<'a>, Option<Span<'a>>)> {
    let mut args = vec![];
    while lex.peek() != &Token::Punct(")") {
        if !matches!(lex.peek(), Token::Ident(_)) {
            let span = lex.advance();
            args.push((FormalArg::Bad(span), None));
            continue;
        }
        let name = lex.advance();
        let arg = FormalArg::Name(name);
        if lex.peek() == &Token::Punct(",") {
            args.push((arg, Some(lex.advance())));
        } else {
            args.push((arg, None));
            break;
        }
    }
    args
}

// Parse a closing token f a pair such as [], () or {}
fn parse_close<'a>(lex: &mut Lex<'a>, closer: &'static str) -> Span<'a> {
    if matches!(lex.peek(), Token::Punct(span) if *span == closer) {
        lex.advance()
    } else {
        // TODO: some error recovery.
        lex.error(lex.span().clone(), format!("Expected {closer}"));
        lex.span()
    }
}

