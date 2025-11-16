use crate::tokeniser::{TokenListIterator};
//use crate::cc_util::{error, errors};
use crate::cc_util;

pub enum NodeKind {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Num(i32), // integer
}

pub struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Node {
        Node {
            kind,
            lhs,
            rhs,
        }
    }
}

pub fn expr(token_iter: &mut TokenListIterator) -> Option<Box<Node>> {
    let mut node: Option<Box<Node>> = mul(token_iter);

    loop {

        if let Ok(_) = token_iter.current().unwrap().expect_symbol("+") {
            token_iter.next();
            node = Some(Box::new(Node::new(NodeKind::Add, node, mul(token_iter))));
            continue;
        }

        if let Ok(_) = token_iter.current().unwrap().expect_symbol("-") {
            token_iter.next();
            node = Some(Box::new(Node::new(NodeKind::Sub, node, mul(token_iter))));
            continue;
        }

        return node;
    }
}

pub fn mul(token_iter: &mut TokenListIterator) -> Option<Box<Node>> {
    let mut node: Option<Box<Node>> = primary(token_iter);

    loop {

        if let Ok(_) = token_iter.current().unwrap().expect_symbol("*") {
            token_iter.next();
            node = Some(Box::new(Node::new(NodeKind::Mul, node, primary(token_iter))));
            continue;
        }

        if let Ok(_) = token_iter.current().unwrap().expect_symbol("/") {
            token_iter.next();
            node = Some(Box::new(Node::new(NodeKind::Div, node, primary(token_iter))));
            continue;
        }

        return node;
    }
}


pub fn primary(token_iter: &mut TokenListIterator) -> Option<Box<Node>> {

    if let Ok(_) = token_iter.current().unwrap().expect_symbol("(") {
        token_iter.next();
        let node = expr(token_iter);

        match token_iter.current().unwrap().expect_symbol(")") {
            Ok(_) => token_iter.next(),
            Err(e) => {
                //cc_util::errors(&[&token_list.origin_formula, &e]);
                cc_util::errors(&["aaa", &e]); // TODO
                return None;
            },
        };

        return node;
    }

    match token_iter.current().unwrap().expect_number() {
        Ok(n) => {
            let node = Some(Box::new(Node::new(NodeKind::Num(n), None, None)));
            token_iter.next();
            return node;
        },
        Err(e) => {
            //cc_util::errors(&[&token_list.origin_formula, &e]);
            cc_util::errors(&["aaa", &e]); // TODO
            return None;
        },
    };
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
        _ => {}
    }

    println!("  push rax");
}
