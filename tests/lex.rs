use sqwipt::lex::{Token, Lex};

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

