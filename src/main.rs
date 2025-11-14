use std::env;
use std::process;

// error helper function
fn error(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1);
}

// the kind of token
#[derive(Debug)]
enum TokenKind {
    Reserved(String), // symbol
    Num(String, i32), // number
    Eof               // the end of input
}

// Token struct
#[derive(Debug)]
struct Token {
    kind: TokenKind,
    next: Option<Box<Token>>
}

impl Token {
    fn new(kind: TokenKind) -> Token {
        Token {
            kind,
            next: None,
        }
    }

    // check the end of eof
    fn at_eof(&self) -> bool {
        match self.kind {
            TokenKind::Eof => true,
            _ => false
        }
    }

    // if TokenKind is Reserved and op is expected, true
    // otherwise, false
    fn consume(&self, op: &str) -> bool {
        match &self.kind {
            TokenKind::Reserved(val) if val == op => {
                true
            },
            _ => false
        }
    }

    // if TokenKind is Reserved and op is expected, OK
    // otherwise, error
    fn expect(&self, op: &str) {
        match &self.kind {
            TokenKind::Reserved(val) if val == op => {
            },
            _ => error(format!("it is not {}", op).as_str())
        }
    }

    // if TokenKind is Num, the value
    // otherwise, error
    fn expect_number<'a>(&'a self) -> &'a i32 {
        match &self.kind {
            TokenKind::Num(_, val) => {
                &val
            },
            _ => error("it is not number")
        }
    }

}

// the list of Token 
struct TokenList {
    head: Option<Box<Token>>
}

// the iterator of the TokenList
struct TokenListIterator<'a> {
    next: Option<&'a Token>,
}

impl TokenList {

    // add new token to the end
    fn push_back(&mut self, new_token: Token) {
        let mut cur_ref = match &mut self.head {
            Some(head_ref) => head_ref,
            None => {
                self.head = Some(Box::new(new_token));
                return;
            }
        };

        while let Some(ref mut next) = cur_ref.next {
            cur_ref = next;
        };

        cur_ref.next = Some(Box::new(new_token));
    }

    // iterator
    fn iter(&self) -> TokenListIterator<'_> {
        TokenListIterator {
            next: self.head.as_deref(),
        }

    }

}

impl<'a> Iterator for TokenListIterator<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|token| {
            self.next = token.next.as_deref();
            token
        })
    }

}


// change input formula into TokenList
fn tokenise(formula: String) -> TokenList {

    let mut token_list = TokenList { head: None };

    // for the number consisting of multiple charactors
    let mut temp: String = String::from("");

    for c in formula.chars() {
        match c {
            ' ' => {
                if !temp.is_empty() {
                    let i = temp.parse::<i32>().unwrap();
                    token_list.push_back(Token::new(TokenKind::Num(temp.to_string(), i)));
                    temp.clear();
                }
            }
            '+'|'-' => {
                if !temp.is_empty() {
                    let i = temp.parse::<i32>().unwrap();
                    token_list.push_back(Token::new(TokenKind::Num(temp.to_string(), i)));
                    temp.clear();
                }
                token_list.push_back(Token::new(TokenKind::Reserved(c.to_string())));
            }
            '0'..'9' => {
                temp = format!("{}{}", temp, c);
            }
            _ => error(format!("Unexpected charactor: {}", c).as_str()),
        }
    }

    if !temp.is_empty() {
        let i = temp.parse::<i32>().unwrap();
        token_list.push_back(Token::new(TokenKind::Num(temp.to_string(), i)));
        temp.clear();
    }

    token_list.push_back(Token::new(TokenKind::Eof));

    token_list
}




fn main() {

    let mut args = env::args();

    if args.len() != 2 {
        error("The number of arguments is wrong");
    }

    args.next();

    let formula = match args.next() {
        Some(arg) => arg,
        None => error("Didn't get a num string"),
    };


    // tokenise
    let token_list = tokenise(formula);

    //for t in token_list.iter() {
    //    println!("----------");
    //    println!("{:?}", t);
    //    println!("{}", t.at_eof());
    //}


    // output assembly
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    print!("  mov rax, ");

    let mut expect_num = true;

    for t in token_list.iter() {
        if t.at_eof() {
            break;
        }

        if expect_num {
            println!("{}", t.expect_number());
            expect_num = false;
        } else {
            if t.consume("+") {
                print!("  add rax, ");
                expect_num = true;
                continue;
            }

            t.expect("-");
            print!("  sub rax, ");
            expect_num = true;
        }
    }

    println!("  ret");
}
