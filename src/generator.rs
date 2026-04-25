use crate::parser::{ Node, NodeKind };

use crate::cc_util::CompileError;

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

    pub fn generate_codes(&mut self, nodes: Vec<Option<Box<Node>>>) -> Result<(), CompileError> {


        // check node
        //println!("{:?}", nodes);
        //println!("-------------------------");



        // output assembly
        println!(".intel_syntax noprefix");

        for nd in nodes {
            // check function definition
            let node = match nd {
                Some(n) => n,
                None => return Ok(()),
            };
            let NodeKind::FuncDef { name, r_type:_, params, stack_size, block } = (*node).kind else {
                return Err(CompileError::new(&["a top-level element must be function definition"]));
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
                if let NodeKind::Lvar { name:_, offset, ty:_ } = (**pn).kind {

                    println!("  mov [rbp-{}], {}", offset, &Self::ARGS_REGISTERS[paramc]);
                    println!();
                    paramc = paramc + 1;
                    if paramc == Self::ARGS_REGISTERS.len() { // only 6 arguments are accepted
                        break;
                    }
                };
            }

            // output body
            self.generate(block)?;
            // TODO need pop rax?

            // output epilogue
            println!(".L.return.{}:", &name);
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
            println!();

        }

        Ok(())
    }

    fn generate(&mut self, nd: Option<Box<Node>>) -> Result<(), CompileError> {

        let node = match nd {
            Some(n) => n,
            None => return Ok(()),
        };

        match (*node).kind {
            NodeKind::If { cond, then, else_then } => {
                self.count = self.count + 1;
                self.generate(cond)?;
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .L.else.{}", self.count);
                self.generate(then)?;
                println!("  jmp .L.end.{}", self.count);
                println!(".L.else.{}:", self.count);
                if let Some(_) = else_then {
                    self.generate(else_then)?;
                }
                println!(".L.end.{}:", self.count);
                println!();
                return Ok(());
            },
            NodeKind::For { init, cond, inc, then } => {
                self.count = self.count + 1;
                self.generate(init)?;
                println!(".L.begin.{}:", self.count);
                if let Some(_) = cond {
                    self.generate(cond)?;
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .L.end.{}", self.count);
                }
                self.generate(then)?;
                if let Some(_) = inc {
                    self.generate(inc)?;
                }
                println!("  jmp .L.begin.{}", self.count);
                println!(".L.end.{}:", self.count);
                println!();
                return Ok(());

            },
            NodeKind::Block { body } => {
                for b in body {
                    self.generate(b)?;
                }
                return Ok(());
            },
            NodeKind::Return { lhs } => {
                self.generate(lhs)?;
                println!("  pop rax");
                println!("  jmp .L.return.{}", &self.cur_func_name);
                println!();
                return Ok(());
            },
            NodeKind::Num { value } => {
                println!("  push {}", value);
                println!();
                return Ok(());
            },
            NodeKind::Lvar { .. } => {
                self.gen_lval(node)?;
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
                println!();
                return Ok(());
            },
            NodeKind::Assign { lhs, rhs } => {
                if let Some(lhs_node) = lhs {
                    self.gen_lval(lhs_node)?;
                };
                self.generate(rhs)?;
                println!("  pop rdi");
                println!("  pop rax");
                println!("  mov [rax], rdi");
                println!("  push rdi");
                println!();
                return Ok(());
            },
            NodeKind::FuncCall { name, args } => {

                let mut argc = 0;

                for arg in args {
                    self.generate(arg)?;
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
                return Ok(());
            },
            NodeKind::Addr { lhs } => {
                // when it is returned from generate_lval, gen_lval, pointer address value stores in rax
                if let Some(lhs_node) = lhs {
                    self.gen_lval(lhs_node)?;
                };
                println!();
                return Ok(());
            },
            NodeKind::Deref { lhs } => {
                self.generate(lhs)?;
                println!("  pop rax");
                println!("  mov rax, [rax]");
                println!("  push rax");
                println!();
                return Ok(());
            },
            _ => {}
        }




        match (*node).kind {
            NodeKind::Add { lhs, rhs } => { 
                self.gen_binary(lhs, rhs)?;
                println!("  add rax, rdi")
            },
            NodeKind::Sub { lhs, rhs } => {
                self.gen_binary(lhs, rhs)?;
                println!("  sub rax, rdi")
            },
            NodeKind::Mul { lhs, rhs } => {
                self.gen_binary(lhs, rhs)?;
                println!("  imul rax, rdi")
            },
            NodeKind::Div { lhs, rhs } => { 
                self.gen_binary(lhs, rhs)?;
                println!("  cqo");
                println!("  idiv rdi");
            },
            NodeKind::Eq { lhs, rhs } => {
                self.gen_binary(lhs, rhs)?;
                println!("  cmp rax, rdi");
                println!("  sete al");
                println!("  movzb rax, al");
            },
            NodeKind::Ne { lhs, rhs } => {
                self.gen_binary(lhs, rhs)?;
                println!("  cmp rax, rdi");
                println!("  setne al");
                println!("  movzb rax, al");
            },
            NodeKind::Lt { lhs, rhs } => {
                self.gen_binary(lhs, rhs)?;
                println!("  cmp rax, rdi");
                println!("  setl al");
                println!("  movzb rax, al");
            },
            NodeKind::Le { lhs, rhs } => {
                self.gen_binary(lhs, rhs)?;
                println!("  cmp rax, rdi");
                println!("  setle al");
                println!("  movzb rax, al");
            },
            _ => {}
        }

        println!("  push rax");
        println!();

        Ok(())
    }

    fn gen_binary(&mut self, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Result<(), CompileError> {

        self.generate(lhs)?;
        self.generate(rhs)?;

        println!("  pop rdi");
        println!("  pop rax");

        Ok(())
    }


    fn gen_lval(&mut self, node: Box<Node>) -> Result<(), CompileError> {
        match (*node).kind {
            NodeKind::Lvar { name:_, offset, ty:_ } => {
                // calcurate local variable address position. so, when this finishes, the top of stack is address value
                println!("  mov rax, rbp");
                println!("  sub rax, {}", offset);
                println!("  push rax");
                println!();

                Ok(())
            },
            NodeKind::Deref { lhs } => {
                // if it is Deref, it is ok to only get the address of lhs using generate, because in Assign, the local variable indicates the address position 
                self.generate(lhs)
            },
            _ => {
                Err(CompileError::new(&["the left value of assign is not a variable."]))
            }
        }

    }
}


