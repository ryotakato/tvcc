mod cc_util;
mod tokeniser;
mod parser;

//use tokeniser::{tokenise};
//use parser::{expr, generate};
//use util::{error, errors};

pub fn compile(formula: String) {

    // tokenise
    let token_list = tokeniser::tokenise(formula);

    //for t in token_list.iter() {
    //    println!("----------");
    //    println!("{:?}", t);
    //    println!("{}", t.at_eof());
    //}

    // create abstract syntax tree (AST)
    let node = parser::expr(&mut token_list.iter());

    // output assembly
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // generate 
    parser::generate(node);

    println!("  pop rax");
    println!("  ret");

}
