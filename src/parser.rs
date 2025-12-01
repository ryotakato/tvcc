use crate::tokeniser::{Token, TokenListIterator, TokenList, TokenKind};
//use crate::cc_util::{error, errors};
use crate::cc_util;

use std::collections::HashMap;

#[derive(Debug)]
pub enum NodeKind {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Eq,  // ==
    Ne,  // !=
    Lt,  // <
    Le,  // <=
    Assign, // =
    Lvar(i32), // local variables + offset
    Num(i32), // integer + value
    Return, // return
    If, // if
    For, // for or while
    Block, // block
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub cond: Option<Box<Node>>,
    pub then: Option<Box<Node>>,
    pub else_then: Option<Box<Node>>,
    pub init: Option<Box<Node>>,
    pub inc: Option<Box<Node>>,
    pub body: Vec<Option<Box<Node>>>,
}

impl Node {
    fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>, 
        cond: Option<Box<Node>>, then: Option<Box<Node>>, else_then: Option<Box<Node>>,
        init: Option<Box<Node>>, inc: Option<Box<Node>>, body: Vec<Option<Box<Node>>>) -> Node {
        Node {
            kind,
            lhs,
            rhs,
            cond,
            then,
            else_then,
            init,
            inc,
            body,
        }
    }
    fn create(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Option<Box<Node>> {
        Some(Box::new(Node::new(kind, lhs, rhs, None, None, None, None, None, Vec::new())))
    }
    fn create_if_node(cond: Option<Box<Node>>, then: Option<Box<Node>>, else_then: Option<Box<Node>>) -> Option<Box<Node>> {
        Some(Box::new(Node { 
            kind: NodeKind::If,
            lhs: None,
            rhs: None,
            cond,
            then,
            else_then,
            init: None,
            inc: None,
            body: Vec::new(),
        }))
    }
    fn create_for_node(init: Option<Box<Node>>, cond: Option<Box<Node>>, inc: Option<Box<Node>>, then: Option<Box<Node>>) -> Option<Box<Node>> {
        Some(Box::new(Node { 
            kind: NodeKind::For,
            lhs: None,
            rhs: None,
            cond,
            then,
            else_then: None,
            init,
            inc,
            body: Vec::new(),
        }))
    }
    fn create_while_node(cond: Option<Box<Node>>, then: Option<Box<Node>>) -> Option<Box<Node>> {
        Some(Box::new(Node { 
            kind: NodeKind::For,
            lhs: None,
            rhs: None,
            cond,
            then,
            else_then: None,
            init: None,
            inc: None,
            body: Vec::new(),
        }))
    }
    fn create_block_node(body: Vec<Option<Box<Node>>>) -> Option<Box<Node>> {
        Some(Box::new(Node { 
            kind: NodeKind::Block,
            lhs: None,
            rhs: None,
            cond: None,
            then: None,
            else_then: None,
            init: None,
            inc: None,
            body,
        }))
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

        let node = Node::create_block_node(stmts);
        return node;
    }

    fn stmt(&mut self) -> Option<Box<Node>> {


        let cur = self.cur_token();

        match cur {
            // "return" ";"
            Token { kind: TokenKind::Return, .. } => {
                let _ = &self.next_token();
                let node = Node::create(NodeKind::Return, self.expr(), None);
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
                        let node = Node::create_if_node(cond, then, else_then);
                        return node;
                    },
                    false => {
                        let node = Node::create_if_node(cond, then, None);
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

                let node = Node::create_for_node(init, cond, inc, then);
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

                let node = Node::create_while_node(cond, then);
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
                                let node = Node::create_block_node(Vec::new());
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
            node = Node::create(NodeKind::Assign, node, self.assign());
            return node
        }

        return node
    }

    fn equality(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.relational();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("==") {
                let _ = &self.next_token();
                node = Node::create(NodeKind::Eq, node, self.relational());
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("!=") {
                let _ = &self.next_token();
                node = Node::create(NodeKind::Ne, node, self.relational());
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
                node = Node::create(NodeKind::Lt, node, self.add());
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("<=") {
                let _ = &self.next_token();
                node = Node::create(NodeKind::Le, node, self.add());
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol(">") {
                let _ = &self.next_token();
                node = Node::create(NodeKind::Lt, self.add(), node);
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol(">=") {
                let _ = &self.next_token();
                node = Node::create(NodeKind::Le, self.add(), node);
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
                node = Node::create(NodeKind::Add, node, self.mul());
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("-") {
                let _ = &self.next_token();
                node = Node::create(NodeKind::Sub, node, self.mul());
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
                node = Node::create(NodeKind::Mul, node, self.unary());
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("/") {
                let _ = &self.next_token();
                node = Node::create(NodeKind::Div, node, self.unary());
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
            let zero = Node::create(NodeKind::Num(0), None, None);
            return Node::create(NodeKind::Sub, zero, self.unary());
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

        if let Ok(lvar) = self.cur_token().expect_ident() {
            let offset = self.local_variable.find_offset(lvar.to_string());
            let node = Node::create(NodeKind::Lvar(offset), None, None);
            let _ = &self.next_token();

            return node;
        }


        match self.cur_token().expect_number() {
            Ok(n) => {
                let node = Node::create(NodeKind::Num(n), None, None);
                let _ = &self.next_token();
                return node;
            },
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
                //return None;
            },
        };
    }
}



