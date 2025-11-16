use crate::util;


// the kind of token
#[derive(Debug)]
pub enum TokenKind {
    Reserved(String), // symbol
    Num(String), // number
    Eof               // the end of input
}

impl TokenKind {
    pub fn num_val(&self) -> Option<i32> {
        match self {
            TokenKind::Num(str) => Some(str.parse::<i32>().unwrap()),
            _ => None,
        }
    }
}

// Token struct
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub next: Option<Box<Token>>,
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

    pub fn next(&self) -> Option<&Token> {
        match &self.next.as_deref() {
            Some(next_t) => {
                //let next_t = *next_t;
                match &next_t.kind {
                    TokenKind::Eof => None,
                    _ => Some(next_t),
                }
            },
            None => None,
        }
    }

    // check the end of eof
    pub fn at_eof(&self) -> bool {
        match self.kind {
            TokenKind::Eof => true,
            _ => false
        }
    }

    // if TokenKind is Reserved and op is expected, Ok
    // otherwise, error string
    pub fn expect(&self, op: &str) -> Result<(), String> {
        match &self.kind {
            TokenKind::Reserved(val) if val == op => Ok(()),
            _ => Err(format!("{:>padding$} it is not {}", '^', op, padding = self.loc+1))
        }
    }

    // if TokenKind is Num, Ok and the value
    // otherwise, error string
    pub fn expect_number(&self) -> Result<i32, String> {
        match &self.kind {
            TokenKind::Num(_) => Ok(self.kind.num_val().unwrap()),
            _ => Err(format!("{:>padding$} it is not number", '^', padding = self.loc+1))
        }
    }

}

// the list of Token 
pub struct TokenList {
    pub head: Option<Box<Token>>,
    pub origin_formula: String
}

// the iterator of the TokenList
pub struct TokenListIterator<'a> {
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
    pub fn iter(&self) -> TokenListIterator<'_> {
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
pub fn tokenise(formula: String) -> TokenList {

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
            '+'|'-'|'*'|'/'|'('|')' => {
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
            _ => util::errors(&[&token_list.origin_formula, format!("{:>padding$} Unexpected charactor", '^', padding = i+1).as_str()])
        }
    }

    if !temp.is_empty() {
        token_list.push_back(Token::new(TokenKind::Num(temp.to_string()), num_loc));
        temp.clear();
    }

    token_list.push_back(Token::new(TokenKind::Eof, 0)); // TODO is the location of eof really 0 ?

    token_list
}

