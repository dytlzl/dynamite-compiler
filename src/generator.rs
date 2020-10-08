use std::io::Write;
use crate::node::{Node, NodeType, Type};
use crate::error::{error, error_at};
use std::collections::VecDeque;

pub struct AsmGenerator<'a> {
    code: &'a str,
    pub buf: Vec<u8>,
    node_stream: &'a Vec<Node>,
    stack_size: usize,
    target_os: Os,
    branch_count: usize,
    loop_stack: VecDeque<usize>,
}

pub enum Os {
    Linux,
    MacOS,
}

const ARGS_REG: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl<'a> AsmGenerator<'a> {
    pub fn new(code: &'a str, node_stream: &'a Vec<Node>, stack_size: usize, target_os: Os) -> Self {
        Self {
            code,
            buf: Vec::new(),
            node_stream,
            stack_size,
            target_os,
            branch_count: 0,
            loop_stack: VecDeque::new(),
        }
    }

    pub fn gen(&mut self) -> std::io::Result<()> {
        writeln!(self.buf, ".intel_syntax noprefix")?;
        writeln!(self.buf)?;
        for node in self.node_stream {
            self.gen_func(node)?;
        }
        Ok(())
    }

    pub fn gen_func(&mut self, node: &Node) -> std::io::Result<()> {
        let prefix = if let Os::MacOS = self.target_os { "_" } else { "" };
        writeln!(self.buf, ".globl {}{}", prefix, node.func_name)?;
        writeln!(self.buf, "{}{}:", prefix, node.func_name)?;

        // prologue
        writeln!(self.buf, "  push rbp")?;
        writeln!(self.buf, "  mov rbp, rsp")?;
        writeln!(self.buf, "  sub rsp, {}", node.offset.unwrap())?;
        for (i, arg) in node.args.iter().enumerate() {
            if let NodeType::LVar = arg.nt {
                writeln!(self.buf, "  mov rax, rbp")?;
                writeln!(self.buf, "  sub rax, {}", arg.offset.unwrap())?;
                writeln!(self.buf, "  mov [rax], {}", ARGS_REG[i])?;
            } else {
                error_at(self.code, arg.token.as_ref().unwrap().pos, "ident expected");
            }
        }
        self.gen_with_node(node.body.as_ref().unwrap())?;
        self.epilogue()?;
        writeln!(self.buf)?;
        Ok(())
    }

    fn epilogue(&mut self) -> std::io::Result<()> {
        writeln!(self.buf, "  mov rsp, rbp")?;
        writeln!(self.buf, "  pop rbp")?;
        writeln!(self.buf, "  ret")?;
        Ok(())
    }

    fn gen_with_node(&mut self, node: &Node) -> std::io::Result<()> {
        match node.nt {
            NodeType::Cf => {
                writeln!(self.buf, "  mov rax, rsp")?;
                writeln!(self.buf, "  add rax, 8")?;
                writeln!(self.buf, "  mov rdi, 16")?;
                writeln!(self.buf, "  cqo\n  idiv rdi")?;
                writeln!(self.buf, "  sub rsp, rdx")?;
                writeln!(self.buf, "  push rdx")?;
                for node in &node.args {
                    self.gen_with_node(node)?;
                }
                for i in 0..node.args.len() {
                    self.pop(ARGS_REG[i])?;
                }
                let prefix = if let Os::MacOS = self.target_os { "_" } else { "" };
                writeln!(self.buf, "  call {}{}", prefix, &node.func_name)?;
                writeln!(self.buf, "  pop rdi")?;
                writeln!(self.buf, "  add rsp, rdi")?;
                self.push("rax")?;
                return Ok(());
            }
            NodeType::If => {
                let branch_num = self.new_branch_num();
                self.gen_with_node(node.cond.as_ref().unwrap())?;
                self.pop_rax()?;
                writeln!(self.buf, "  cmp rax, 0")?;
                writeln!(self.buf, "  je .Lelse{}", branch_num)?;
                self.gen_with_node(node.then.as_ref().unwrap())?;
                writeln!(self.buf, "  jmp .Lend{}", branch_num)?;
                writeln!(self.buf, ".Lelse{}:", branch_num)?;
                if let Some(_) = node.els {
                    self.gen_with_node(node.els.as_ref().unwrap())?;
                }
                writeln!(self.buf, ".Lend{}:", branch_num)?;
                return Ok(());
            }
            NodeType::Whl => {
                let branch_num = self.new_branch_num();
                self.loop_stack.push_back(branch_num);
                writeln!(self.buf, ".Lbegin{}:", branch_num)?;
                self.reset_stack()?;
                self.gen_with_node(node.cond.as_ref().unwrap())?;
                self.pop_rax()?;
                writeln!(self.buf, "  cmp rax, 0")?;
                writeln!(self.buf, "  je .Lend{}", branch_num)?;
                self.gen_with_node(node.then.as_ref().unwrap())?;
                writeln!(self.buf, "  jmp .Lbegin{}", branch_num)?;
                writeln!(self.buf, ".Lend{}:", branch_num)?;
                self.loop_stack.pop_back();
                return Ok(());
            }
            NodeType::For => {
                let branch_num = self.new_branch_num();
                self.loop_stack.push_back(branch_num);
                if let Some(_) = node.ini {
                    self.gen_with_node(node.ini.as_ref().unwrap())?;
                    self.pop_rax()?;
                }
                writeln!(self.buf, ".Lbegin{}:", branch_num)?;
                self.reset_stack()?;
                if let Some(_) = node.cond {
                    self.gen_with_node(node.cond.as_ref().unwrap())?;
                    self.pop_rax()?;
                } else {
                    writeln!(self.buf, "  mov rax, 1")?;
                }
                writeln!(self.buf, "  cmp rax, 0")?;
                writeln!(self.buf, "  je .Lend{}", branch_num)?;
                self.gen_with_node(node.then.as_ref().unwrap())?;
                if let Some(_) = node.upd {
                    self.gen_with_node(node.upd.as_ref().unwrap())?;
                    self.pop_rax()?;
                }
                writeln!(self.buf, "  jmp .Lbegin{}", branch_num)?;
                writeln!(self.buf, ".Lend{}:", branch_num)?;
                self.loop_stack.pop_back();
                return Ok(());
            }
            NodeType::Block => {
                self.gen_with_vec(&node.children)?;
                return Ok(());
            }
            NodeType::Brk => {
                if let Some(branch_num) = self.loop_stack.back() {
                    writeln!(self.buf, "  jmp .Lend{}", branch_num)?;
                } else {
                    error_at(self.code, node.token.as_ref().unwrap().pos, "unexpected break found");
                }
                return Ok(());
            }
            NodeType::Ret => {
                self.gen_with_node(node.lhs.as_ref().unwrap())?;
                self.pop_rax()?;
                self.epilogue()?;
                return Ok(());
            }
            NodeType::Num => {
                self.push(node.value.unwrap())?;
                return Ok(());
            }
            NodeType::LVar => {
                self.gen_addr(node)?;
                if let Some(Type::Arr(_, _)) = node.cty {
                    return Ok(());
                }
                self.pop_rax()?;
                writeln!(self.buf, "  mov rax, [rax]")?;
                self.push("rax")?;
                return Ok(());
            }
            NodeType::Addr => {
                self.gen_addr(node.lhs.as_ref().unwrap())?;
                return Ok(());
            }
            NodeType::Deref => {
                self.gen_with_node(node.lhs.as_ref().unwrap())?;
                self.pop_rax()?;
                writeln!(self.buf, "  mov rax, [rax]")?;
                self.push("rax")?;
                return Ok(());
            }
            NodeType::Asg => {
                self.gen_addr(node.lhs.as_ref().unwrap())?;
                self.gen_with_node(node.rhs.as_ref().unwrap())?;
                self.pop_rdi()?;
                self.pop_rax()?;
                writeln!(self.buf, "  mov [rax], rdi")?;
                self.push("rdi")?;
                return Ok(());
            }
            _ => {}
        }
        self.gen_with_node(node.lhs.as_ref().unwrap())?;
        self.gen_with_node(node.rhs.as_ref().unwrap())?;
        self.pop_rdi()?;
        self.pop_rax()?;
        match node.nt {
            NodeType::Add => {
                if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                    writeln!(self.buf, "  imul rdi, {}", t.size_of())?;
                } else if let Some(t) = node.rhs.as_ref().unwrap().dest_type() {
                    writeln!(self.buf, "  imul rax, {}", t.size_of())?;
                }
                writeln!(self.buf, "  add rax, rdi")?;
            }
            NodeType::Sub => {
                if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                    writeln!(self.buf, "  imul rdi, {}", t.size_of())?;
                }
                writeln!(self.buf, "  sub rax, rdi")?;
            }
            NodeType::Mul => {
                writeln!(self.buf, "  imul rax, rdi")?;
            }
            NodeType::Div => {
                writeln!(self.buf, "  cqo\n  idiv rdi")?;
            }
            NodeType::Mod => {
                writeln!(self.buf, "  cqo\n  idiv rdi")?;
                self.push("rdx")?;
                return Ok(());
            }
            NodeType::Eq | NodeType::Ne | NodeType::Lt | NodeType::Le => {
                writeln!(
                    self.buf,
                    "  cmp rax, rdi\n  {} al\n  {} rax, al",
                    match node.nt {
                        NodeType::Eq => "sete",
                        NodeType::Ne => "setne",
                        NodeType::Lt => "setl",
                        NodeType::Le => "setle",
                        _ => unreachable!()
                    },
                    if let Os::MacOS = self.target_os { "movzx" } else { "movzb" })?;
            }
            _ => {
                error("unexpected node");
            }
        }
        self.push("rax")?;
        Ok(())
    }

    fn gen_with_vec(&mut self, v: &Vec<Node>) -> std::io::Result<()> {
        for node in v {
            self.gen_with_node(node)?;
            self.pop_rax()?;
            self.reset_stack()?;
        }
        Ok(())
    }

    fn gen_addr(&mut self, node: &Node) -> std::io::Result<()> {
        match node.nt {
            NodeType::LVar => {
                writeln!(self.buf, "  mov rax, rbp")?;
                writeln!(self.buf, "  sub rax, {}", node.offset.unwrap())?;
                writeln!(self.buf, "  push rax")?
            }
            NodeType::Deref => {
                self.gen_with_node(node.lhs.as_ref().unwrap())?;
            }
            _ => {
                unreachable!();
            }
        }
        Ok(())
    }

    fn new_branch_num(&mut self) -> usize {
        self.branch_count += 1;
        self.branch_count
    }

    fn pop_rax(&mut self) -> std::io::Result<()> {
        writeln!(self.buf, "  pop rax")?;
        Ok(())
    }

    fn pop_rdi(&mut self) -> std::io::Result<()> {
        writeln!(self.buf, "  pop rdi")?;
        Ok(())
    }

    fn pop<T: std::fmt::Display>(&mut self, v: T) -> std::io::Result<()> {
        writeln!(self.buf, "  pop {}", v)?;
        Ok(())
    }

    fn push<T: std::fmt::Display>(&mut self, v: T) -> std::io::Result<()> {
        writeln!(self.buf, "  push {}", v)?;
        Ok(())
    }

    fn reset_stack(&mut self) -> std::io::Result<()> {
        writeln!(self.buf, "  mov rsp, rbp")?;
        writeln!(self.buf, "  sub rsp, {}", self.stack_size)?;
        Ok(())
    }

    fn _printf(&mut self, offset: usize) -> std::io::Result<()> {
        self.reset_stack()?;
        writeln!(self.buf, r#"  lea rdi, [rip + L_.str]
  mov esi, dword ptr [rbp - {}]
  mov al, 0
  call _printf"#, offset)?;
        Ok(())
    }

    fn _set_string(&mut self) -> std::io::Result<()> {
        writeln!(self.buf, r#"  .section __TEXT,__cstring,cstring_literals
L_.str:                                 ## @.str
  .asciz "value = %d\n""#).unwrap();
        Ok(())
    }
}