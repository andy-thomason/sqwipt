use crate::lex::{Lex, Token};

#[derive(Debug, PartialEq, Clone)]
pub struct Scope<'a> {
    items: Vec<Item<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Item<'a> {
    Def(&'a str, Function<'a>),
    Let(&'a str, Expr<'a>),
    Expr(Expr<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FormalArg<'a> {
    name: &'a str,
    default: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function<'a> {
    formal_args: Vec<FormalArg<'a>>,
    scope: Scope<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    Ident(&'a str),
    Int(&'a str),
    Float(&'a str),
    Hex(&'a str),
    Str(&'a str),
    Array(&'a str, Vec<Expr<'a>>, &'a str),
    Binary(Box<Expr<'a>>, &'a str, Box<Expr<'a>>),
    Unary(&'a str, Box<Expr<'a>>),
    Index(Box<Expr<'a>>, &'a str, Box<Expr<'a>>, &'a str),
    Dot(Box<Expr<'a>>, &'a str, Box<Expr<'a>>),
    Bad(&'a str),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    span: &'a str,
    reason: &'static str,
}

pub trait Parse<'a> : Sized {
    // Return either Some(item) or None if it cannot be one from
    // the first token.
    //
    // If there is a subsequent error, report it but still return
    // The type.

    fn parse(lex: &'a mut Lex) -> Option<Self>;
}

impl<'a> Parse<'a> for Expr<'a> {
    fn parse(lex: &'a mut Lex) -> Option<Self> {
        // println!("Expr::parse {:?}", lex.peek());
        match lex.peek() {
            Token::Keyword(_) |
            Token::Punct(_) |
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

impl<'a> Parse<'a> for Item<'a> {
    // Return either Some(item) or None if it cannot be one from
    // the first token.
    // If there is a subsequent error, report it but still return an item.
    fn parse(lex: &'a mut Lex) -> Option<Self> {
        // println!("Item::parse {:?}", lex.peek());
        match lex.peek() {
            // These tokens start statements.
            Token::Keyword(token) => {
                match token {
                    // "def" => Some(Item::Def(Def::parse(lex)),
                    // "if" => Some(Item::If(If::parse(lex)),
                    // "for" => Some(Item::For(For::parse(lex)),
                    _ => Some(Item::Expr(Expr::parse(lex)?))
                }
            }

            // These tokens start an expression.
            Token::Punct(_) |
            Token::Int(_) |
            Token::Float(_) |
            Token::Hex(_) |
            Token::Ident(_) |
            Token::Str(_) => {
                Some(Item::Expr(Expr::parse(lex)?))
            }
            Token::Newline(_) => {
                lex.next();
                Item::parse(lex)
            }

            // These can't start an item.
            Token::Begin(_) => None,
            Token::End(_) => None,
            Token::Eof(_) => None,
            Token::UnknownToken(_) => None,
            Token::UnterminatedString(_) => None,
        }
    }
}

/// A binop is a series of atoms joined by binary operators +, -, * etc.
fn parse_binop<'l, 'a>(lex: &'l mut Lex<'a>, min_precidence: usize) -> Expr<'a> {
    // println!("parse_binop {:?}", lex.peek());
    let mut lhs = parse_atom(lex);
    loop {
        let precidence = match lex.peek() {
            Token::Punct("+") => 30,
            Token::Punct("*") => 40,
            _ => break lhs,
        };

        if precidence < min_precidence {
            break lhs;
        } else {
            let span = lex.advance();
            let rhs = parse_binop(lex, precidence-1);
            lhs = Expr::Binary(Box::new(lhs), span, Box::new(rhs));
        }
    }

}

// An atom is the bread in the binop sandwich.
// 1, fred, "xyz", +1, fred[2], fred(1, 2, 3), [1, 2, 3], (1, 2, 3), (1+2)
fn parse_atom<'l, 'a>(lex: &'l mut Lex<'a>) -> Expr<'a> {
    println!("parse_atom {:?}", lex.peek());
    let prefix = match lex.peek() {
        Token::Punct("!") |
        Token::Punct("+") |
        Token::Punct("-") => {
            Expr::Unary(lex.advance(), Box::new(parse_atom(lex)))
        }
        Token::Punct("(") => {
            let span1 = lex.advance();
            let expr = Expr::parse(lex)
            Expr::Paren(lex.advance(), parse_)
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
            lex.error(span, format!("parse_atom"));
            return Expr::Bad(span)
        }
    };
    match lex.peek() {
        Token::Punct("[") => {
            let lspan = lex.advance();
            let expr = parse_binop(lex, usize::MAX);
            let rspan = lex.span();
            if let Token::Punct("]") = lex.peek() {
                lex.advance();
            } else {
                lex.error(rspan, format!("Expected ]"));
            }
            Expr::Index(Box::new(prefix), lspan, Box::new(expr), rspan)
        }
        _ => prefix
    }
}
