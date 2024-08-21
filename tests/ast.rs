use sqwipt::{ast::{Expr, Parse}, lex::{Lex, Token}};

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

    expr!("a", r#"Some(Ident("a"))"#);

    expr!("!1", r#"Some(Unary("!", Int("1")))"#);
    expr!("+1", r#"Some(Unary("+", Int("1")))"#);
    expr!("-1", r#"Some(Unary("-", Int("1")))"#);

    expr!("(1)", "Some(Paren(\"(\", Int(\"1\"), \")\"))");
    expr!("(1,)", "Some(Tuple(\"(\", [(Int(\"1\"), Some(\",\"))], \")\"))");

    expr!("1 + 2 * 3", r#"Some(Binary(Int("1"), "+", Binary(Int("2"), "*", Int("3"))))"#);
    expr!("1 * 2 + 3", r#"Some(Binary(Binary(Int("1"), "*", Int("2")), "+", Int("3")))"#);
    expr!("1 + 2 + 3", "Some(Binary(Binary(Int(\"1\"), \"+\", Int(\"2\")), \"+\", Int(\"3\")))");

    expr!("1[2]", r#"Some(Index(Int("1"), "[", Int("2"), "]"))"#);
    expr!("1(2)", r#"Some(Call(Int("1"), "(", [(Int("2"), None)], ")"))"#);
    expr!("1(2)(3)", r#"Some(Call(Call(Int("1"), "(", [(Int("2"), None)], ")"), "(", [(Int("3"), None)], ")"))"#);
    expr!("1[2](3)", r#"Some(Call(Index(Int("1"), "[", Int("2"), "]"), "(", [(Int("3"), None)], ")"))"#);
    expr!("(1)(2)", "Some(Call(Paren(\"(\", Int(\"1\"), \")\"), \"(\", [(Int(\"2\"), None)], \")\"))");

    expr!("|x| x + 1", "Some(Closure(Closure { open: \"|\", formal_args: [(Name(\"x\"), None)], close: \"|\", body: Binary(Ident(\"x\"), \"+\", Int(\"1\")) }))");
    expr!("\n  1", "Some(Block(Block { begin: \"\", items: [Int(\"1\")], end: \"\" }))");
    expr!("\n  1\n  2", "Some(Block(Block { begin: \"\", items: [Int(\"1\"), Int(\"2\")], end: \"\" }))");
    expr!("\n  1\n  (\n    2\n  )", "Some(Block(Block { begin: \"\", items: [Int(\"1\"), Paren(\"(\", Block(Block { begin: \"\", items: [Int(\"2\")], end: \"\" }), \")\")], end: \"\" }))");
}

#[test]
fn test_expr_bad() {
    macro_rules! expr {
        ($s: expr, $res: expr, $next: expr) => {
            println!("expr bad test [{}]", $s);
            let mut lex = Lex::new($s);
            let item = Expr::parse(&mut lex);
            assert_eq!(format!("{item:?}"), $res);
            let next_token = lex.peek();
            assert_eq!(format!("{next_token:?}"), $next);
        };
    }

    expr!("1", r#"Some(Int("1"))"#, r#"Eof("")"#);
    expr!("1 2", r#"Some(Int("1"))"#, r#"Int("2")"#);
    expr!("$", r#"None"#, r#"UnknownToken("$")"#);
}

// #[test]
// fn test_def() {
//     macro_rules! item {
//         ($s: expr, $res: expr, $next: expr) => {
//             println!("test_def [{}]", $s);
//             let mut lex = Lex::new($s);
//             let item = Item::parse(&mut lex);
//             assert_eq!(format!("{item:?}"), $res);
//             let next_token = lex.peek();
//             assert_eq!(format!("{next_token:?}"), $next);
//         };
//     }

//     item!("1", r#"Some(Expr { expr: Int("1") })"#, r#"Eof("")"#);

//     item!("def fred(a): a+1", "Some(Def { def: \"def\", ident: \"fred\", function: Function { open: \"(\", formal_args: [(Name(\"a\"), None)], close: \")\", colon: \":\", scope: Scope { items: [Expr { expr: Binary(Ident(\"a\"), \"+\", Int(\"1\")) }] } } })", r#"Eof("")"#);
//     item!("def fred(a):\n  a+1", "Some(Def { def: \"def\", ident: \"fred\", function: Function { open: \"(\", formal_args: [(Name(\"a\"), None)], close: \")\", colon: \":\", scope: Scope { items: [Expr { expr: Binary(Ident(\"a\"), \"+\", Int(\"1\")) }] } } })", r#"Eof("")"#);
//     item!("def fred(a):\n  a+1\n  a+2", "Some(Def { def: \"def\", ident: \"fred\", function: Function { open: \"(\", formal_args: [(Name(\"a\"), None)], close: \")\", colon: \":\", scope: Scope { items: [Expr { expr: Binary(Ident(\"a\"), \"+\", Int(\"1\")) }, Expr { expr: Binary(Ident(\"a\"), \"+\", Int(\"2\")) }] } } })", r#"Eof("")"#);
// }
