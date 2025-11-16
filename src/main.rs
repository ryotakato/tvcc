use std::env;

mod util;

use tvcc::compile;



fn main() {

    let mut args = env::args();

    if args.len() != 2 {
        util::error("The number of arguments is wrong");
    }

    args.next();

    let formula = match args.next() {
        Some(arg) => arg,
        None => util::error("Didn't get a num string"),
    };

    tvcc::compile(formula);
}
