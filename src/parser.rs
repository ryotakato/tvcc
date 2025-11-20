use crate::tokeniser::{Token, TokenListIterator,TokenList};
//use crate::cc_util::{error, errors};
use crate::cc_util;

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
}

pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
}

impl Node {
    fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Node {
        Node {
            kind,
            lhs,
            rhs,
        }
    }
    fn create(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Option<Box<Node>> {
        Some(Box::new(Node::new(kind, lhs, rhs)))
    }
}

pub struct Parser<'a> {
    token_iter: TokenListIterator<'a>,
    origin_formula: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(token_list: &'a TokenList) -> Parser<'a> {

        let token_iter: TokenListIterator<'a> = token_list.iter();
        let origin_formula = &token_list.origin_formula;

        Parser {
            token_iter,
            origin_formula,
        }
    }

    pub fn parse(&mut self) -> Vec<Option<Box<Node>>> {
        self.program()
    }

    fn cur_token(&self) -> &Token {
        &self.token_iter.current().unwrap()
    }

    fn next_token(&mut self) -> &'a Token {
        &self.token_iter.next().unwrap()
    }

    fn program(&mut self) -> Vec<Option<Box<Node>>> {
        let mut stmts: Vec<Option<Box<Node>>> = Vec::new();

        while !self.cur_token().at_eof() {
            stmts.push(self.stmt());
        }

        stmts
    }

    fn stmt(&mut self) -> Option<Box<Node>> {
        let node: Option<Box<Node>> = self.expr();

        match self.cur_token().expect_symbol(";") {
            Ok(_n) => {
                let _ = &self.next_token();
                return node;
            },
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
                //return None;
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
            let offset = ((lvar.chars().next().unwrap() as i32) - ('a' as i32) + 1) * 8;
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



