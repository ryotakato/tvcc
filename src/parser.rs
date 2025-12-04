use crate::tokeniser::{Token, TokenListIterator, TokenList, TokenKind};
//use crate::cc_util::{error, errors};
use crate::cc_util;

use std::collections::HashMap;

#[derive(Debug)]
pub enum Node {
    Add { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // +
    Sub { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // -
    Mul { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // *
    Div { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // /
    Eq { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // ==
    Ne { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // !=
    Lt { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // <
    Le { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // <=
    Assign { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // =
    Lvar { offset: i32 }, // local variables + offset
    Num {value: i32 }, // integer + value
    Return { lhs: Option<Box<Node>> }, // return
    If { cond: Option<Box<Node>>, then: Option<Box<Node>>, else_then: Option<Box<Node>> }, // if
    For { init: Option<Box<Node>>, cond: Option<Box<Node>>, inc: Option<Box<Node>>, then: Option<Box<Node>>}, // for or while
    Block { body: Vec<Option<Box<Node>>> }, // block
    FuncCall { name: String, args: Vec<Option<Box<Node>>> }, // func call
}

impl Node {
    fn wrap(self) -> Option<Box<Node>> {
        Some(Box::new(self))
    }
}


// struct for local variales 
struct LocalVariable {
    latest_offset: i32,
    variables: HashMap<String, i32>,
}

impl LocalVariable {
    fn new() -> LocalVariable {
        LocalVariable {
            latest_offset: 0,
            variables: HashMap::new(),
        }
    }

    fn find_offset(&mut self, variale_name: String) -> i32 {
        self.variables.entry(variale_name).or_insert_with(|| {self.latest_offset = self.latest_offset + 8; self.latest_offset}).clone()
    }
}



pub struct Parser<'a> {
    token_iter: TokenListIterator<'a>,
    origin_formula: &'a str,
    local_variable: LocalVariable,
}

impl<'a> Parser<'a> {
    pub fn new(token_list: &'a TokenList) -> Parser<'a> {

        let token_iter: TokenListIterator<'a> = token_list.iter();
        let origin_formula = &token_list.origin_formula;
        let local_variable = LocalVariable::new();

        Parser {
            token_iter,
            origin_formula,
            local_variable,
        }
    }

    pub fn parse(&mut self) -> Vec<Option<Box<Node>>> {
        self.stmt_expect_symbol("{");

        let mut program: Vec<Option<Box<Node>>> = Vec::new();
        program.push(self.compound_stmt());

        if !self.cur_token().at_eof() {
            cc_util::errors(&[&self.origin_formula, "The last } is unexpected"]);
        }

        program
    }

    fn cur_token(&self) -> &Token {
        &self.token_iter.current().unwrap()
    }

    fn next_token(&mut self) -> &'a Token {
        &self.token_iter.next().unwrap()
    }

    //fn program(&mut self) -> Vec<Option<Box<Node>>> {
    //    let mut stmts: Vec<Option<Box<Node>>> = Vec::new();

    //    while !self.cur_token().at_eof() {
    //        stmts.push(self.stmt());
    //    }

    //    stmts
    //}

    fn compound_stmt(&mut self) -> Option<Box<Node>> {

        let mut stmts: Vec<Option<Box<Node>>> = Vec::new();
        while let Err(_) = self.cur_token().expect_symbol("}") {
            stmts.push(self.stmt());
        }
        self.stmt_expect_symbol("}");

        let node = Node::Block { body: stmts, }.wrap();
        return node;
    }

    fn stmt(&mut self) -> Option<Box<Node>> {


        let cur = self.cur_token();

        match cur {
            // "return" ";"
            Token { kind: TokenKind::Return, .. } => {
                let _ = &self.next_token();
                let node = Node::Return { lhs: self.expr(), }.wrap();
                self.stmt_expect_symbol(";");
                return node;
            },
            // "if" "(" expr ")" stmt ("else" stmt)?
            Token { kind: TokenKind::If, .. } => {


                let _ = &self.next_token();
                self.stmt_expect_symbol("(");
                // cond
                let cond = self.expr();

                self.stmt_expect_symbol(")");

                // then
                let then = self.stmt();

                // else_then
                match self.cur_token().at_else() {
                    true => {
                        let _ = &self.next_token();
                        let else_then = self.stmt();
                        let node = Node::If { cond, then, else_then }.wrap();
                        return node;
                    },
                    false => {
                        let node = Node::If { cond, then, else_then: None }.wrap();
                        return node;
                    }
                }

            },
            // "for" "(" expr? ";" expr? ";" expr? ")" stmt
            Token { kind: TokenKind::For, .. } => {

                let _ = &self.next_token();
                self.stmt_expect_symbol("(");

                // init
                let init = match self.cur_token().expect_symbol(";") {
                    Ok(_n) => None,
                    Err(_) => self.expr(),
                };

                // cond
                let _ = &self.next_token();
                let cond = match self.cur_token().expect_symbol(";") {
                    Ok(_n) => None,
                    Err(_) => self.expr(),
                };

                // inc
                let _ = &self.next_token();
                let inc = match self.cur_token().expect_symbol(")") {
                    Ok(_n) => None,
                    Err(_) => self.expr(),
                };

                // then
                let _ = &self.next_token();
                let then = self.stmt();

                let node = Node::For { init, cond, inc, then }.wrap();
                return node;

            },
            // "while" "(" expr ")" stmt
            Token { kind: TokenKind::While, .. } => {

                let _ = &self.next_token();
                self.stmt_expect_symbol("(");
                // cond
                let cond = self.expr();

                self.stmt_expect_symbol(")");

                // then
                let then = self.stmt();

                let node = Node::For { init: None, cond, inc: None, then }.wrap();
                return node;
            },
            _ => {
                match cur.expect_symbol("{") {
                    // "{" compound_stmt
                    Ok(_) => {
                        let _ = &self.next_token();
                        let node = self.compound_stmt();
                        return node;
                    },
                    // expr ";"
                    Err(_) => {
                        match cur.expect_symbol(";") {
                            Ok(_) => {
                                let _ = &self.next_token();
                                let node = Node::Block { body: Vec::new() }.wrap();
                                return node;
                            },
                            Err(_) => {
                                let node = self.expr();
                                self.stmt_expect_symbol(";");
                                return node;
                            }
                        }
                    }
                }
            }
        }
    }

    fn stmt_expect_symbol(&mut self, symbol: &str) {

        match self.cur_token().expect_symbol(symbol) {
            Ok(_n) => {
                let _ = &self.next_token();
            },
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
            },
        };
    }



    fn expr(&mut self) -> Option<Box<Node>> {
        self.assign()
    }

    fn assign(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.equality();

        if let Ok(_) = self.cur_token().expect_symbol("=") {
            let _ = &self.next_token();
            node = Node::Assign { lhs: node, rhs: self.assign(), }.wrap();
            return node
        }

        return node
    }

    fn equality(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.relational();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("==") {
                let _ = &self.next_token();
                node = Node::Eq { lhs: node, rhs: self.relational(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("!=") {
                let _ = &self.next_token();
                node = Node::Ne { lhs: node, rhs: self.relational(), }.wrap();
                continue;
            }

            return node;
        }
    }

    fn relational(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.add();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("<") {
                let _ = &self.next_token();
                node = Node::Lt { lhs: node, rhs: self.add(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("<=") {
                let _ = &self.next_token();
                node = Node::Le { lhs: node, rhs: self.add(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol(">") {
                let _ = &self.next_token();
                node = Node::Lt { lhs: self.add(), rhs: node, }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol(">=") {
                let _ = &self.next_token();
                node = Node::Le { lhs: self.add(), rhs: node, }.wrap();
                continue;
            }

            return node;
        }
    }

    fn add(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.mul();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("+") {
                let _ = &self.next_token();
                node = Node::Add { lhs: node, rhs: self.mul(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("-") {
                let _ = &self.next_token();
                node = Node::Sub { lhs: node, rhs: self.mul(), }.wrap();
                continue;
            }

            return node;
        }
    }

    fn mul(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.unary();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("*") {
                let _ = &self.next_token();
                node = Node::Mul { lhs: node, rhs: self.unary(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("/") {
                let _ = &self.next_token();
                node = Node::Div { lhs: node, rhs: self.unary(), }.wrap();
                continue;
            }

            return node;
        }
    }

    fn unary(&mut self) -> Option<Box<Node>> {
        if let Ok(_) = self.cur_token().expect_symbol("+") {
            let _ = &self.next_token();
            return self.unary();
        }
        if let Ok(_) = self.cur_token().expect_symbol("-") {
            let _ = &self.next_token();
            let zero = Node::Num { value: 0, }.wrap();
            return Node::Sub { lhs: zero, rhs: self.unary(), }.wrap();
        }

        return self.primary();
    }


    fn primary(&mut self) -> Option<Box<Node>> {

        if let Ok(_) = self.cur_token().expect_symbol("(") {
            let _ = &self.next_token();
            let node = self.expr();

            match self.cur_token().expect_symbol(")") {
                Ok(_) => &self.next_token(),
                Err(e) => {
                    cc_util::errors(&[&self.origin_formula, &e]);
                    //return None;
                },
            };

            return node;
        }

        if let Ok(name) = self.cur_token().expect_ident() {
            let name = name.to_string();
            let _ = &self.next_token();

            let node = match self.cur_token().expect_symbol("(") {
                Ok(_) => {
                    // func call
                    self.func_call(name)
                },
                Err(_) => {
                    // local variable
                    let offset = self.local_variable.find_offset(name);
                    Node::Lvar{ offset }.wrap()
                }
            };

            return node;
        }


        match self.cur_token().expect_number() {
            Ok(n) => {
                let node = Node::Num { value: n }.wrap();
                let _ = &self.next_token();
                return node;
            },
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
                //return None;
            },
        };
    }



    fn func_call(&mut self, name: String) -> Option<Box<Node>> {

        let _ = &self.next_token();

        let mut args: Vec<Option<Box<Node>>> = Vec::new();

        while let Err(_) = self.cur_token().expect_symbol(")") {
            args.push(self.assign());

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }
        }

        let _ = &self.next_token(); // skip ")"

        Node::FuncCall { name, args, }.wrap()
    }
}



