use crate::cc_util;

// the kind of token
#[derive(Debug)]
pub enum TokenKind {
    Reserved(String), // symbol
    Ident(String),    // identifier
    Num(String),      // number
    Return,           // return
    If,               // if
    Else,             // else
    While,            // while
    For,              // for
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

    // check the end of eof
    pub fn at_eof(&self) -> bool {
        match self.kind {
            TokenKind::Eof => true,
            _ => false
        }
    }

    // check return
    pub fn at_return(&self) -> bool {
        match self.kind {
            TokenKind::Return => true,
            _ => false
        }
    }

    // check else
    pub fn at_else(&self) -> bool {
        match self.kind {
            TokenKind::Else => true,
            _ => false
        }
    }

    // if TokenKind is Reserved and op is expected, Ok
    // otherwise, error string
    pub fn expect_symbol(&self, op: &str) -> Result<(), String> {
        match &self.kind {
            TokenKind::Reserved(val) if val == op => Ok(()),
            _ => Err(format!("{:>padding$} expected {}", '^', op, padding = self.loc+1))
        }
    }

    // if TokenKind is Ident, Ok
    // otherwise, error string
    pub fn expect_ident(&self) -> Result<&str, String> {
        match &self.kind {
            TokenKind::Ident(val) => Ok(val),
            _ => Err(format!("{:>padding$} expected an ident", '^', padding = self.loc+1))
        }
    }

    // if TokenKind is Num, Ok and the value
    // otherwise, error string
    pub fn expect_number(&self) -> Result<i32, String> {
        match &self.kind {
            TokenKind::Num(_) => Ok(self.kind.num_val().unwrap()),
            _ => Err(format!("{:>padding$} expected a number", '^', padding = self.loc+1))
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

impl<'a> TokenListIterator<'a> {
    pub fn current(&self) -> Option<&'a Token> {
        self.next
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


pub struct Tokeniser {
    formula: String,
}

impl Tokeniser {
    pub fn new(formula: String) -> Tokeniser {
        Tokeniser {
            formula,
        }
    }

    // change input formula into TokenList
    pub fn tokenise(&self) -> TokenList {

        let mut token_list = TokenList { head: None, origin_formula: self.formula.clone() };

        let mut i = 0;
        // for the number consisting of multiple charactors

        let mut start_loc = 0;
        let mut start_num_flag = false;
        let mut start_identifier_flag = false;


        let len = self.formula.len();

        loop {

            if len <= i {
                break;
            }

            // if identifier continue
            if start_identifier_flag {
                match &self.formula[i..i+1].chars().next().unwrap() {
                    ('0'..='9')|('a'..='z')|('A'..='Z')|'_' => {
                        i = i+1;
                        continue;
                    },
                    _ => {
                        let name = &self.formula[start_loc..i];
                        match name {
                            "return" => token_list.push_back(Token::new(TokenKind::Return, start_loc)),
                            "if" => token_list.push_back(Token::new(TokenKind::If, start_loc)),
                            "else" => token_list.push_back(Token::new(TokenKind::Else, start_loc)),
                            "while" => token_list.push_back(Token::new(TokenKind::While, start_loc)),
                            "for" => token_list.push_back(Token::new(TokenKind::For, start_loc)),
                            _ => token_list.push_back(Token::new(TokenKind::Ident(name.to_string()), start_loc)),
                        }
                        start_identifier_flag = false;
                        start_loc = 0;
                    }
                }

            }

            // if number continue
            if start_num_flag {
                match &self.formula[i..i+1].chars().next().unwrap() {
                    '0'..='9' => {
                        i = i+1;
                        continue;
                    },
                    _ => {
                        token_list.push_back(Token::new(TokenKind::Num(self.formula[start_loc..i].to_string()), start_loc));
                        start_num_flag = false;
                        start_loc = 0;
                    }
                }

            }

            // empty
            if &self.formula[i..i+1] == " " {
                i = i+1;
                continue;
            }

            // 2 bytes char
            if i+2 <= len {
                match &self.formula[i..i+2] {
                    "=="|"!="|"<="|">=" => {
                        token_list.push_back(Token::new(TokenKind::Reserved(self.formula[i..i+2].to_string()), i));
                        i = i+2;
                        continue;
                    },
                    _ => {}
                }
            }

            // 1 byte char
            match &self.formula[i..i+1].chars().next().unwrap() {
                '+'|'-'|'*'|'/'|'('|')'|'<'|'>'|'='|';'|'{'|'}' => {
                    token_list.push_back(Token::new(TokenKind::Reserved(self.formula[i..i+1].to_string()), i));
                    i = i+1;
                    continue;
                },
                ('a'..='z')|('A'..='Z')|'_' => {
                    start_identifier_flag = true;
                    start_loc = i;
                    i = i+1;
                    continue;
                },
                '0'..='9' => {
                    start_num_flag = true;
                    start_loc = i;
                    i = i+1;
                    continue;
                },
                _ => {}
            }


            cc_util::errors(&[&token_list.origin_formula, format!("{:>padding$} Unexpected charactor", '^', padding = i+1).as_str()])

        }

        if start_identifier_flag {
            token_list.push_back(Token::new(TokenKind::Ident(self.formula[start_loc..i].to_string()), start_loc));
            //start_identifier_flag = false;
            //start_loc = 0;
        }
        if start_num_flag {
            token_list.push_back(Token::new(TokenKind::Num(self.formula[start_loc..i].to_string()), start_loc));
            //start_num_flag = false;
            //start_loc = 0;
        }

        token_list.push_back(Token::new(TokenKind::Eof, len));

        token_list
    }
}



