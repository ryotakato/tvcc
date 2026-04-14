use std::env;

use tvcc;

mod cc_util;
use cc_util::CompileError;


fn main() {

    let program = get_program(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        panic!();
    });

    // TODO twice call clone, weird
    tvcc::compile(program.clone()).unwrap_or_else(|err| {
        eprintln!("{}", program.clone());
        eprintln!("{}", err);
        panic!();
    });
}



fn get_program(mut args: env::Args) -> Result<String, CompileError> {

    if args.len() != 2 {
        return Err(CompileError::new(&["The number of arguments is wrong"]));
    }

    args.next();

    return match args.next() {
        Some(arg) => Ok(arg),
        None => Err(CompileError::new(&["Didn't get a program"])),
    };
}
