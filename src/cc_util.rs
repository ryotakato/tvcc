use std::process;

// error helper function
pub fn error(message: &str) -> ! {
    println!("");
    eprintln!("{}", message);
    process::exit(1);
}

pub fn errors(messages: &[&str]) -> ! {
    println!("");
    for m in messages {
        eprintln!("{}", m);
    };

    process::exit(1);
}
