use crate::parser::{Node, NodeKind};
use crate::cc_util;

pub struct Generator {
    count: usize
}


impl Generator {

    pub fn new() -> Generator {
        Generator {
            count: 0
        }
    }

    pub fn generate_nodes(&mut self, nodes: Vec<Option<Box<Node>>>) {
        for nd in nodes {
            self.generate(nd);

            println!("  pop rax");
            println!();
        }
    }

    pub fn generate(&mut self, nd: Option<Box<Node>>) {

        let node = match nd {
            Some(n) => n,
            None => return,
        };

        match node.kind {
            NodeKind::If => {
                self.count = self.count + 1;
                self.generate(node.cond);
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .L.else.{}", self.count);
                self.generate(node.then);
                println!("  jmp .L.end.{}", self.count);
                println!(".L.else.{}:", self.count);
                if let Some(_) = node.else_then {
                    self.generate(node.else_then);
                }
                println!(".L.end.{}:", self.count);
                println!();
                return;
            },
            NodeKind::For => {
                self.count = self.count + 1;
                self.generate(node.init);
                println!(".L.begin.{}:", self.count);
                if let Some(_) = node.cond {
                    self.generate(node.cond);
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .L.end.{}", self.count);
                }
                self.generate(node.then);
                if let Some(_) = node.inc {
                    self.generate(node.inc);
                }
                println!("  jmp .L.begin.{}", self.count);
                println!(".L.end.{}:", self.count);
                println!();
                return;

            },
            NodeKind::Block => {
                for b in node.body {
                    self.generate(b);
                }
                return;
            },
            NodeKind::Return => {
                self.generate(node.lhs);
                println!("  pop rax");
                println!("  mov rsp, rbp");
                println!("  pop rbp");
                println!("  ret");
                println!();
                return;
            },
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
            NodeKind::FuncCall => {
                println!("  mov rax, 0");
                println!("  call {}", &node.func_name);
                println!("  push rax");
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


