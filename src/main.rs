use std::env;

mod cc_util;

use tvcc;



fn main() {

    let mut args = env::args();

    if args.len() != 2 {
        cc_util::error("The number of arguments is wrong");
    }

    args.next();

    let formula = match args.next() {
        Some(arg) => arg,
        None => cc_util::error("Didn't get a num string"),
    };

    tvcc::compile(formula);
}
