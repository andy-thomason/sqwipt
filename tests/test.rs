use sqwipt::{ast::{Expr, Parse}, lex::{Lex, Token}};

#[test]
fn test_lex() {
    macro_rules! lex {
        ($s : expr, $tok : expr) => {
            let mut lex = Lex::new($s);
            assert_eq!(lex.peek(), &$tok);
            lex.advance();
            assert!(matches!(lex.peek(), Token::Eof(_)));
        }
    }
    lex!("1", Token::Int("1"));
    lex!("1.0", Token::Float("1.0"));
    lex!("0x12abcd", Token::Hex("0x12abcd"));
    lex!("\"xyz\"", Token::Str("\"xyz\""));
    lex!("'xyz'", Token::Str("'xyz'"));
    lex!(r#"'xyz\''"#, Token::Str(r#"'xyz\''"#));
    lex!("\"xyz", Token::UnterminatedString("\"xyz"));
    // let mut lex = Lex::new(r#"
    //     struct Bert a b c d

    //     trait Jim
    //         fn fred a b
    //         fn bert c
        
    //     impl Jim for Fred
    //         fn fred a b:
    //             a + b
    //         fn a self:
    //             self.a

    //     impl Bert
    //         fn new is Bert 1 2 3 4

    //     x = 1

    //     fn fred a b
    //         a + b

    //     "#);
    
    // loop {
    //     let t = lex.next();
    //     eprintln!("{t:?}");
    //     if let Token::Eof(_) = t {
    //         break;
    //     }
    // }
}

#[test]
fn test_expr() {
    macro_rules! expr {
        ($s: expr, $res: expr) => {
            println!("expr test [{}]", $s);
            let mut lex = Lex::new($s);
            let item = Expr::parse(&mut lex);
            assert_eq!(format!("{item:?}"), $res);
            assert!(matches!(lex.peek(), Token::Eof(_)));
        };
    }

    expr!("1", r#"Some(Int("1"))"#);
    expr!("1.0", r#"Some(Float("1.0"))"#);
    expr!("1.0e10", r#"Some(Float("1.0e10"))"#);
    expr!(r#""1""#, r#"Some(Str("\"1\""))"#);

    expr!("!1", r#"Some(Unary("!", Int("1")))"#);
    expr!("+1", r#"Some(Unary("+", Int("1")))"#);
    expr!("-1", r#"Some(Unary("-", Int("1")))"#);

    expr!("(1)", r#"Some(Unary("-", Int("1")))"#);

    expr!("1 + 2 * 3", r#"Some(Binary(Int("1"), "+", Binary(Int("2"), "*", Int("3"))))"#);
    expr!("1 * 2 + 3", r#"Some(Binary(Binary(Int("1"), "*", Int("2")), "+", Int("3")))"#);
    expr!("1 + 2 + 3", r#"Some(Binary(Int("1"), "+", Binary(Int("2"), "+", Int("3"))))"#);
}
