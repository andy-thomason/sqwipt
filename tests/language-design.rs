use std::str::FromStr;

use sqwipt::{ast::parse_programme, lex::Lex};


#[test]
fn language_design() {
    for f in [
        "hello-world.sqw"
    ] {
        let mut p = std::path::PathBuf::from_str("tests/language-design").unwrap();
        p.push(f);
        let src = std::fs::read_to_string(p).unwrap();
        let lex = &mut Lex::new(&src);
        use sqwipt::ast::Programme::*;
        match parse_programme(lex) {
            Bad() => panic!("bad programme {f}"),
            Good(exprs) => {
                for _expr in exprs {
                    // expr.execute();
                }
            }
        }
    }
}