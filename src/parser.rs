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
    Num(i32), // integer
}

pub struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
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

    pub fn parse(&mut self) -> Option<Box<Node>> {
        self.expr()
    }

    fn cur_token(&self) -> &Token {
        &self.token_iter.current().unwrap()
    }

    fn next_token(&mut self) -> &'a Token {
        &self.token_iter.next().unwrap()
    }

    fn expr(&mut self) -> Option<Box<Node>> {
        self.equality()
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
                    return None;
                },
            };

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
                return None;
            },
        };
    }
}





pub fn generate(nd: Option<Box<Node>>) {
    let node = match nd {
        Some(n) => n,
        None => return,
    };

    if let NodeKind::Num(n) = node.kind {
        println!("  push {}", n);
        return;
    }

    generate(node.lhs);
    generate(node.rhs);

    println!("  pop rdi");
    println!("  pop rax");

    match node.kind {
        NodeKind::Add => println!("  add rax, rdi"),
        NodeKind::Sub => println!("  sub rax, rdi"),
        NodeKind::Mul => println!("  imul rax, rdi"),
        NodeKind::Div => { 
            println!("  cqo");
            println!("  idiv rdi");
        },
        NodeKind::Eq => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        },
        NodeKind::Ne => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        },
        NodeKind::Lt => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        },
        NodeKind::Le => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        },
        _ => {}
    }

    println!("  push rax");
}
