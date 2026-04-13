use crate::parser::{ Node };
use crate::cc_util;

pub struct Generator {
    count: usize,
    cur_func_name: String,
}


impl Generator {

    const ARGS_REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

    pub fn new() -> Generator {
        Generator {
            count: 0,
            cur_func_name: String::from(""),
        }
    }

    pub fn generate_codes(&mut self, nodes: Vec<Option<Box<Node>>>) {


        // check node
        //println!("{:?}", nodes);
        //println!("-------------------------");



        // output assembly
        println!(".intel_syntax noprefix");

        for nd in nodes {
            // check function definition
            let node = match nd {
                Some(n) => n,
                None => return,
            };
            let Node::FuncDef { name, r_type, params, stack_size, block } = *node else {
                cc_util::error("a top-level element must be function definition");
            };

            self.cur_func_name = name.to_string();

            // output func area
            println!(".globl {}", &name);
            println!("{}:", &name);
            println!();

            // output prologue
            println!("  push rbp");
            println!("  mov rbp, rsp");
            println!("  sub rsp, {}", stack_size);
            println!();

            // output params
            let mut paramc = 0;

            for param in &params {

                let pn = match param {
                    Some(n) => n,
                    None => continue,
                };
                if let Node::Lvar { offset } = **pn {

                    println!("  mov [rbp-{}], {}", offset, &Self::ARGS_REGISTERS[paramc]);
                    println!();
                    paramc = paramc + 1;
                    if paramc == Self::ARGS_REGISTERS.len() { // only 6 arguments are accepted
                        break;
                    }
                };
            }

            // output body
            self.generate(block);
            // TODO need pop rax?

            // output epilogue
            println!(".L.return.{}:", &name);
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
            println!();

        }
    }

    fn generate(&mut self, nd: Option<Box<Node>>) {

        let node = match nd {
            Some(n) => n,
            None => return,
        };

        match *node {
            Node::If { cond, then, else_then } => {
                self.count = self.count + 1;
                self.generate(cond);
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .L.else.{}", self.count);
                self.generate(then);
                println!("  jmp .L.end.{}", self.count);
                println!(".L.else.{}:", self.count);
                if let Some(_) = else_then {
                    self.generate(else_then);
                }
                println!(".L.end.{}:", self.count);
                println!();
                return;
            },
            Node::For { init, cond, inc, then } => {
                self.count = self.count + 1;
                self.generate(init);
                println!(".L.begin.{}:", self.count);
                if let Some(_) = cond {
                    self.generate(cond);
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .L.end.{}", self.count);
                }
                self.generate(then);
                if let Some(_) = inc {
                    self.generate(inc);
                }
                println!("  jmp .L.begin.{}", self.count);
                println!(".L.end.{}:", self.count);
                println!();
                return;

            },
            Node::Block { body } => {
                for b in body {
                    self.generate(b);
                }
                return;
            },
            Node::Return { lhs } => {
                self.generate(lhs);
                println!("  pop rax");
                println!("  jmp .L.return.{}", &self.cur_func_name);
                println!();
                return;
            },
            Node::Num { value } => {
                println!("  push {}", value);
                println!();
                return;
            },
            Node::Lvar { .. } => {
                self.gen_lval(node);
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
                println!();
                return;
            },
            Node::Assign { lhs, rhs } => {
                self.generate_lval(lhs);
                self.generate(rhs);
                println!("  pop rdi");
                println!("  pop rax");
                println!("  mov [rax], rdi");
                println!("  push rdi");
                println!();
                return;
            },
            Node::FuncCall { name, args } => {

                let mut argc = 0;

                for arg in args {
                    self.generate(arg);
                    argc = argc + 1;
                    if argc == Self::ARGS_REGISTERS.len() { // only 6 arguments are accepted
                        break;
                    }
                }

                for i in (0..argc).rev() { // reverse, because stack is FIFO
                    println!("  pop {}", &Self::ARGS_REGISTERS[i]);
                }

                println!("  mov rax, 0");
                println!("  call {}", &name);
                println!("  push rax");
                println!();
                return;
            },
            Node::Addr { lhs } => {
                // when it is returned from generate_lval, gen_lval, pointer address value stores in rax
                self.generate_lval(lhs);
                println!();
                return;
            },
            Node::Deref { lhs } => {
                self.generate(lhs);
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
                println!();
                return;
            },
            _ => {}
        }




        match *node {
            Node::Add { lhs, rhs } => { 
                self.gen_binary(lhs, rhs);
                println!("  add rax, rdi")
            },
            Node::Sub { lhs, rhs } => {
                self.gen_binary(lhs, rhs);
                println!("  sub rax, rdi")
            },
            Node::Mul { lhs, rhs } => {
                self.gen_binary(lhs, rhs);
                println!("  imul rax, rdi")
            },
            Node::Div { lhs, rhs } => { 
                self.gen_binary(lhs, rhs);
                println!("  cqo");
                println!("  idiv rdi");
            },
            Node::Eq { lhs, rhs } => {
                self.gen_binary(lhs, rhs);
                println!("  cmp rax, rdi");
                println!("  sete al");
                println!("  movzb rax, al");
            },
            Node::Ne { lhs, rhs } => {
                self.gen_binary(lhs, rhs);
                println!("  cmp rax, rdi");
                println!("  setne al");
                println!("  movzb rax, al");
            },
            Node::Lt { lhs, rhs } => {
                self.gen_binary(lhs, rhs);
                println!("  cmp rax, rdi");
                println!("  setl al");
                println!("  movzb rax, al");
            },
            Node::Le { lhs, rhs } => {
                self.gen_binary(lhs, rhs);
                println!("  cmp rax, rdi");
                println!("  setle al");
                println!("  movzb rax, al");
            },
            _ => {}
        }

        println!("  push rax");
        println!();
    }

    fn gen_binary(&mut self, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) {

        self.generate(lhs);
        self.generate(rhs);

        println!("  pop rdi");
        println!("  pop rax");
    }



    fn generate_lval(&mut self, nd: Option<Box<Node>>) {
        let node = match nd {
            Some(n) => n,
            None => return,
        };

        self.gen_lval(node);
    }

    fn gen_lval(&mut self, node: Box<Node>) {
        match *node {
            Node::Lvar { offset } => {
                // calcurate local variable address position. so, when this finishes, the top of stack is address value
                println!("  mov rax, rbp");
                println!("  sub rax, {}", offset);
                println!("  push rax");
                println!();
            },
            Node::Deref { lhs } => {
                // if it is Deref, it is ok to only get the address of lhs using generate, because in Assign, the local variable indicates the address position 
                self.generate(lhs);
            },
            _ => {
                cc_util::error("the left value of assign is not a variable.");
            }
        }

    }
}


