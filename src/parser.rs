use crate::tokeniser::{Token, TokenList, TokenKind};
use crate::util;

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

pub fn expr(mut token: &Token) -> Option<Box<Node>> {
    //println!("{:?}", token);
    let mut node: Option<Box<Node>> = mul(token);


    loop {

        println!("=================");
        println!("expr => {:?}", token);
        if let Some(nt) = &token.next() {

            if let Ok(_) = nt.expect("+") {
                token = nt;
                if let Some(nt) = &token.next() {
                    token = nt;
                    node = Some(Box::new(Node::new(NodeKind::Add, node, mul(token))));
                    continue;
                }
            }

            if let Ok(_) = nt.expect("-") {
                token = nt;
                if let Some(nt) = &token.next() {
                    token = nt;
                    node = Some(Box::new(Node::new(NodeKind::Sub, node, mul(token))));
                    continue;
                }
            }
        }

        return node;
    }
}

pub fn mul(mut token: &Token) -> Option<Box<Node>> {
    let mut node: Option<Box<Node>> = primary(token);


    loop {

        println!("=================");
        println!("mul => {:?}", token);
        if let Some(nt) = &token.next() {

            if let Ok(_) = nt.expect("*") {
                token = nt;
                if let Some(nt) = &token.next() {
                    token = nt;
                    node = Some(Box::new(Node::new(NodeKind::Mul, node, primary(token))));
                    continue;
                }
            }

            if let Ok(_) = nt.expect("/") {
                token = nt;
                if let Some(nt) = &token.next() {
                    token = nt;
                    node = Some(Box::new(Node::new(NodeKind::Div, node, primary(token))));
                    continue;
                }
            }
        }

        return node;
    }
}


pub fn primary(mut token: &Token) -> Option<Box<Node>> {
    let mut node: Option<Box<Node>> = None;

    println!("=================");
    println!("primary => {:?}", token);

    if let Ok(_) = token.expect("(") {
        if let Some(nt) = &token.next() {
            token = nt;

            node = expr(token);

            if let Ok(_) = token.expect(")") {
                // TODO not error, otherwise error
            }

            return node;
        }

    }

    if let Ok(n) = token.expect_number() {
        node = Some(Box::new(Node::new(NodeKind::Num(n), None, None)));
        if let Some(nt) = &token.next() {
            token = nt;
        }
        return node;
    }
    
    return None;
}


pub fn generate(node: &Node) {
    if let NodeKind::Num(n) = node.kind {
        println!("  push {}", n);
        return;
    }

    if let Some(l) = &node.lhs {
        generate(&l);
    }
    if let Some(r) = &node.rhs {
        generate(&r);
    }

    println!("  pop rdi");
    println!("  pop rax");

    match node.kind {
        NodeKind::Add => println!("  add rax, rdi"),
        NodeKind::Sub => println!("  sub rax, rdi"),
        NodeKind::Mul => println!("  mul rax, rdi"),
        NodeKind::Div => { 
            println!("  cqo");
            println!("  idiv rdi");
        },
        _ => {}
    }

    println!("  push rax");
}
