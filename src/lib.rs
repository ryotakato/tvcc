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

    // generate 
    Generator::new().generate_codes(nodes);
}
