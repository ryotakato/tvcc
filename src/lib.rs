mod cc_util;
mod tokeniser;
mod parser;
mod generator;

//use tokeniser::{tokenise};
//use parser::{expr, generate};
use parser::Parser;
use tokeniser::Tokeniser;
use generator::Generator;
//use util::{error, errors};

pub fn compile(formula: String) {

    // tokenise
    let token_list = Tokeniser::new(formula).tokenise();


    // create abstract syntax tree (AST)
    let nodes = Parser::new(&token_list).parse();

    // output assembly
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!();



    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");
    println!();

    // generate 
    Generator::new().generate_nodes(nodes);

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");

}
