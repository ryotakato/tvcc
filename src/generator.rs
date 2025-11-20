use crate::parser::{Node, NodeKind};
use crate::cc_util;

pub struct Generator {
}


impl Generator {

    pub fn new() -> Generator {
        Generator {}
    }

    pub fn generate_nodes(&self, nodes: Vec<Option<Box<Node>>>) {
        for nd in nodes {
            self.generate(nd);

            println!("  pop rax");
            println!();
        }
    }

    pub fn generate(&self, nd: Option<Box<Node>>) {
        let node = match nd {
            Some(n) => n,
            None => return,
        };

        match node.kind {
            NodeKind::Num(n) => {
                println!("  push {}", n);
                println!();
                return;
            },
            NodeKind::Lvar(_) => {
                self.gen_lval(node);
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
                println!();
                return;
            },
            NodeKind::Assign => {
                self.generate_lval(node.lhs);
                self.generate(node.rhs);
                println!("  pop rdi");
                println!("  pop rax");
                println!("  mov [rax], rdi");
                println!("  push rdi");
                println!();
                return;

            },
            _ => {}
        }





        self.generate(node.lhs);
        self.generate(node.rhs);

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
        println!();
    }




    fn generate_lval(&self, nd: Option<Box<Node>>) {
        let node = match nd {
            Some(n) => n,
            None => return,
        };

        self.gen_lval(node);
    }

    fn gen_lval(&self, node: Box<Node>) {
        match node.kind {
            NodeKind::Lvar(offset) => {
                println!("  mov rax, rbp");
                println!("  sub rax, {}", offset);
                println!("  push rax");
                println!();
            },
            _ => {
                cc_util::error("the left value of assign is not a variable.");
            }
        }

    }
}


