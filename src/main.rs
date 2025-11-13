use std::env;
use std::process;

fn main() {

    let mut args = env::args();

    if args.len() != 2 {
        eprintln!("The number of arguments is wrong");
        process::exit(1);
    }

    args.next();

    let formula = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("Didn't get a num string");
            process::exit(1);
        }
    };

    let mut dividec_formula: Vec<String> = vec![];
    let mut temp: String = String::from("");
    for c in formula.chars() {
        match c {
            '+'|'-' => {
                if !temp.is_empty() {
                    dividec_formula.push(temp.to_string());
                    temp.clear();
                }
                dividec_formula.push(c.to_string());
            }
            '0'..'9' => {
                temp = format!("{}{}", temp, c);
            }
            _ => {
                eprintln!("Unexpected charactor: {}", c);
                process::exit(1);
            }
        }

    }

    if !temp.is_empty() {
        dividec_formula.push(temp.to_string());
        temp.clear();
    }

    //println!("{:?}", dividec_formula);



    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    print!("  mov rax, ");

    for f in dividec_formula {
        match f.as_str() {
            "+" => print!("  add rax, "),
            "-" => print!("  sub rax, "),
            _ => println!("{}", f),
        };
    }

    println!("  ret");
}
