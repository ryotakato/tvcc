use std::env;
use std::process;

fn main() {

    let mut args = env::args();

    if args.len() != 2 {
        eprintln!("The number of arguments is wrong");
        process::exit(1);
    }

    args.next();

    let num: i32 = match args.next() {
        Some(arg) => {
            match arg.parse::<i32>() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("The argument is not number");
                    process::exit(1);
                }
            }
        },
        None => {
            eprintln!("Didn't get a num string");
            process::exit(1);
        }
    };

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", num);
    println!("  ret");
}
