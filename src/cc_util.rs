//use std::process;
use std::fmt;
use std::error::Error;

//// error helper function
//pub fn error(message: &str) -> ! {
//    println!("");
//    eprintln!("{}", message);
//    process::exit(1);
//}
//
//pub fn errors(messages: &[&str]) -> ! {
//    println!("");
//    for m in messages {
//        eprintln!("{}", m);
//    };
//
//    process::exit(1);
//}


#[derive(Debug)]
pub struct CompileError {
    pub messages: Vec<String>,
}

impl CompileError {
    pub fn new(messages: &[&str]) -> CompileError {
        CompileError {
            messages: messages.iter().map(|&s| s.to_string()).collect()
        }
    }
}

impl Error for CompileError {}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for m in &self.messages {
            writeln!(f, "{}", m)?
        };
        Ok(())
    }
}
