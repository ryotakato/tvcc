mod cc_util;
mod tokeniser;
mod parser;
mod generator;

use parser::Parser;
use tokeniser::Tokeniser;
use generator::Generator;

use cc_util::CompileError;

pub fn compile(formula: String) -> Result<(), CompileError> {

    // tokenise
    let token_list = Tokeniser::new(formula).tokenise()?;

    // create abstract syntax tree (AST)
    let nodes = Parser::new(&token_list).parse()?;

    // generate 
    Generator::new().generate_codes(nodes)
}

