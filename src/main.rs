use std::env;
use std::process;

// error helper function
fn error(message: &str) -> ! {
    println!("");
    eprintln!("{}", message);
    process::exit(1);
}

fn errors(messages: &[&str]) -> ! {
    println!("");
    for m in messages {
        eprintln!("{}", m);
    }
    process::exit(1);
}



// the kind of token
#[derive(Debug)]
enum TokenKind {
    Reserved(String), // symbol
    Num(String), // number
    Eof               // the end of input
}

impl TokenKind {
    fn num_val(&self) -> Option<i32> {
        match self {
            TokenKind::Num(str) => Some(str.parse::<i32>().unwrap()),
            _ => None,
        }
    }
}

// Token struct
#[derive(Debug)]
struct Token {
    kind: TokenKind,
    next: Option<Box<Token>>,
    loc: usize,
}

impl Token {
    fn new(kind: TokenKind, loc: usize) -> Token {
        Token {
            kind,
            next: None,
            loc,
        }
    }

    // check the end of eof
    fn at_eof(&self) -> bool {
        match self.kind {
            TokenKind::Eof => true,
            _ => false
        }
    }

    // if TokenKind is Reserved and op is expected, Ok
    // otherwise, error string
    fn expect(&self, op: &str) -> Result<(), String> {
        match &self.kind {
            TokenKind::Reserved(val) if val == op => Ok(()),
            _ => Err(format!("{:>padding$} it is not {}", '^', op, padding = self.loc+1))
        }
    }

    // if TokenKind is Num, Ok and the value
    // otherwise, error string
    fn expect_number(&self) -> Result<i32, String> {
        match &self.kind {
            TokenKind::Num(_) => Ok(self.kind.num_val().unwrap()),
            _ => Err(format!("{:>padding$} it is not number", '^', padding = self.loc+1))
        }
    }

}

// the list of Token 
struct TokenList {
    head: Option<Box<Token>>,
    origin_formula: String
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

    let mut token_list = TokenList { head: None, origin_formula: formula.clone() };

    // for the number consisting of multiple charactors
    let mut temp: String = String::from("");
    let mut num_loc = 0;

    for (i, c) in formula.chars().enumerate() {
        match c {
            ' ' => {
                if !temp.is_empty() {
                    token_list.push_back(Token::new(TokenKind::Num(temp.to_string()), num_loc));
                    temp.clear();
                }
            }
            '+'|'-' => {
                if !temp.is_empty() {
                    token_list.push_back(Token::new(TokenKind::Num(temp.to_string()), num_loc));
                    temp.clear();
                }
                token_list.push_back(Token::new(TokenKind::Reserved(c.to_string()), i));
            }
            '0'..'9' => {
                num_loc = i;
                temp = format!("{}{}", temp, c);
            }
            _ => errors(&[&token_list.origin_formula, format!("{:>padding$} Unexpected charactor", '^', padding = i+1).as_str()])
        }
    }

    if !temp.is_empty() {
        token_list.push_back(Token::new(TokenKind::Num(temp.to_string()), num_loc));
        temp.clear();
    }

    token_list.push_back(Token::new(TokenKind::Eof, 0)); // TODO is the location of eof really 0 ?

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
            let number = match t.expect_number() {
                Ok(n) => n,
                Err(e) => {
                    errors(&[&token_list.origin_formula, &e]);
                },
            };
            println!("{}", number);
            expect_num = false;
        } else {
            let plus_result = t.expect("+");
            match plus_result {
                Ok(_) => {
                    print!("  add rax, ");
                    expect_num = true;
                    continue;
                },
                Err(_) => {}
            }

            let minus_result = t.expect("-");
            match minus_result {
                Ok(_) => {
                    print!("  sub rax, ");
                    expect_num = true;
                },
                Err(e) => {
                    errors(&[&token_list.origin_formula, &e]);
                }
            }
        }
    }

    println!("  ret");
}
