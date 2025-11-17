mod cc_util;
mod tokeniser;
mod parser;

//use tokeniser::{tokenise};
//use parser::{expr, generate};
use parser::Parser;
use tokeniser::Tokeniser;
//use util::{error, errors};

pub fn compile(formula: String) {

    // tokenise
    let token_list = Tokeniser::new(formula).tokenise();


    // create abstract syntax tree (AST)
    let node = Parser::new(&token_list).parse();

    // output assembly
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // generate 
    parser::generate(node);

    println!("  pop rax");
    println!("  ret");

}
