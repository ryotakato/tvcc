mod util;
mod tokeniser;
mod parser;

use tokeniser::{tokenise};
use parser::{expr, generate};
use util::{error, errors};

pub fn compile(formula: String) {

    // tokenise
    let token_list = tokeniser::tokenise(formula);

    //for t in token_list.iter() {
    //    println!("----------");
    //    println!("{:?}", t);
    //    println!("{}", t.at_eof());
    //}

    let node = parser::expr(token_list.head.as_deref().unwrap());


    let node = node.unwrap(); // TODO

    // output assembly
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    parser::generate(&node);


    //let mut expect_num = true;

    //for t in token_list.iter() {
    //    if t.at_eof() {
    //        break;
    //    }

    //    if expect_num {
    //        let number = match t.expect_number() {
    //            Ok(n) => n,
    //            Err(e) => {
    //                util::errors(&[&token_list.origin_formula, &e]);
    //            },
    //        };
    //        println!("{}", number);
    //        expect_num = false;
    //    } else {
    //        let plus_result = t.expect("+");
    //        match plus_result {
    //            Ok(_) => {
    //                print!("  add rax, ");
    //                expect_num = true;
    //                continue;
    //            },
    //            Err(_) => {}
    //        }

    //        let minus_result = t.expect("-");
    //        match minus_result {
    //            Ok(_) => {
    //                print!("  sub rax, ");
    //                expect_num = true;
    //            },
    //            Err(e) => {
    //                util::errors(&[&token_list.origin_formula, &e]);
    //            }
    //        }
    //    }
    //}


    println!("  pop rax");
    println!("  ret");

}
