use std::io::Write;
use crate::node::{Node, NodeType};
use crate::error::{error, error_at};
use std::collections::VecDeque;

pub struct AsmGenerator<'a> {
    code: &'a str,
    pub buf: Vec<u8>,
    node_stream: &'a Vec<Node>,
    target_os: Os,
    branch_count: usize,
    loop_stack: VecDeque<usize>,
}

pub enum Os {
    Linux,
    MacOS,
}

impl<'a> AsmGenerator<'a> {
    pub fn new(code: &'a str, node_stream: &'a Vec<Node>, target_os: Os) -> Self {
        Self {
            code,
            buf: Vec::new(),
            node_stream,
            target_os,
            branch_count: 0,
            loop_stack: VecDeque::new(),
        }
    }

    pub fn gen_asm(&mut self, stack_size: usize) -> std::io::Result<()> {
        writeln!(self.buf, ".intel_syntax noprefix")?;
        let entry_point = if let Os::MacOS = self.target_os { "_main" } else { "main" };
        writeln!(self.buf, ".globl {}", entry_point)?;
        writeln!(self.buf, "{}:", entry_point)?;

        // prologue
        writeln!(self.buf, "  push rbp")?;
        writeln!(self.buf, "  mov rbp, rsp")?;
        writeln!(self.buf, "  sub rsp, {}", stack_size)?;

        for node in self.node_stream {
            self.gen_asm_with_node(node)?;
        }
        self.pop_rax()?;
        self.epilogue()?;
        Ok(())
    }

    fn epilogue(&mut self) -> std::io::Result<()> {
        writeln!(self.buf, "  mov rsp, rbp")?;
        writeln!(self.buf, "  pop rbp")?;
        writeln!(self.buf, "  ret")?;
        Ok(())
    }

    fn gen_asm_with_local_variable(&mut self, n: &Node) -> std::io::Result<()> {
        writeln!(self.buf, "  mov rax, rbp")?;
        assert_ne!(n.offset, 0);
        writeln!(self.buf, "  sub rax, {}", n.offset)?;
        writeln!(self.buf, "  push rax")?;
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
    fn push_value(&mut self, s: &str) -> std::io::Result<()> {
        writeln!(self.buf, "  push {}", s)?;
        Ok(())
    }

    fn gen_asm_with_node(&mut self, n: &Node) -> std::io::Result<()> {
        match n.nt {
            NodeType::If => {
                let branch_num = self.new_branch_num();
                self.gen_asm_with_node(n.cond.as_ref().unwrap())?;
                self.pop_rax()?;
                writeln!(self.buf, "  cmp rax, 0")?;
                writeln!(self.buf, "  je .Lelse{}", branch_num)?;
                self.gen_asm_with_node(n.then.as_ref().unwrap())?;
                writeln!(self.buf, "  jmp .Lend{}", branch_num)?;
                writeln!(self.buf, ".Lelse{}:", branch_num)?;
                if let Some(_) = n.els {
                    self.gen_asm_with_node(n.els.as_ref().unwrap())?;
                }
                writeln!(self.buf, ".Lend{}:", branch_num)?;
                return Ok(())
            }
            NodeType::Whl => {
                let branch_num = self.new_branch_num();
                self.loop_stack.push_back(branch_num);
                writeln!(self.buf, ".Lbegin{}:", branch_num)?;
                self.gen_asm_with_node(n.cond.as_ref().unwrap())?;
                self.pop_rax()?;
                writeln!(self.buf, "  cmp rax, 0")?;
                writeln!(self.buf, "  je .Lend{}", branch_num)?;
                self.gen_asm_with_node(n.then.as_ref().unwrap())?;
                writeln!(self.buf, "  jmp .Lbegin{}", branch_num)?;
                writeln!(self.buf, ".Lend{}:", branch_num)?;
                self.loop_stack.pop_back();
                return Ok(())
            }
            NodeType::For => {
                let branch_num = self.new_branch_num();
                self.loop_stack.push_back(branch_num);
                if let Some(_) = n.ini {
                    self.gen_asm_with_node(n.ini.as_ref().unwrap())?;
                    self.pop_rax()?;
                }
                writeln!(self.buf, ".Lbegin{}:", branch_num)?;
                if let Some(_) = n.cond {
                    self.gen_asm_with_node(n.cond.as_ref().unwrap())?;
                    self.pop_rax()?;
                } else {
                    writeln!(self.buf, "  mov rax, 1")?;
                }
                writeln!(self.buf, "  cmp rax, 0")?;
                writeln!(self.buf, "  je .Lend{}", branch_num)?;
                self.gen_asm_with_node(n.then.as_ref().unwrap())?;
                if let Some(_) = n.upd {
                    self.gen_asm_with_node(n.upd.as_ref().unwrap())?;
                    self.pop_rax()?;
                }
                writeln!(self.buf, "  jmp .Lbegin{}", branch_num)?;
                writeln!(self.buf, ".Lend{}:", branch_num)?;
                self.loop_stack.pop_back();
                return Ok(())
            }
            NodeType::Block => {
                for child in &n.children {
                    self.gen_asm_with_node(child)?;
                }
                return Ok(())
            }
            NodeType::Brk => {
                if let Some(branch_num) = self.loop_stack.back() {
                    writeln!(self.buf, "  jmp .Lend{}", branch_num)?;
                } else {
                    error_at(self.code, n.token.as_ref().unwrap().pos, "unexpected break found");
                }
                return Ok(());
            }
            NodeType::Ret => {
                self.gen_asm_with_node(n.lhs.as_ref().unwrap())?;
                self.pop_rax()?;
                self.epilogue()?;
                return Ok(());
            }
            NodeType::Num => {
                writeln!(self.buf, "  push {}", n.value)?;
                return Ok(());
            }
            NodeType::LVar => {
                self.gen_asm_with_local_variable(n)?;
                self.pop_rax()?;
                writeln!(self.buf, "  mov rax, [rax]")?;
                self.push_value("rax")?;
                return Ok(())
            }
            NodeType::Asg => {
                self.gen_asm_with_local_variable(n.lhs.as_ref().unwrap())?;
                self.gen_asm_with_node(n.rhs.as_ref().unwrap())?;
                self.pop_rdi()?;
                self.pop_rax()?;
                writeln!(self.buf, "  mov [rax], rdi")?;
                self.push_value("rdi")?;
                return Ok(())
            }
            _ => {}
        }
        self.gen_asm_with_node(n.lhs.as_ref().unwrap())?;
        self.gen_asm_with_node(n.rhs.as_ref().unwrap())?;
        self.pop_rdi()?;
        self.pop_rax()?;
        match n.nt {
            NodeType::Add => {
                writeln!(self.buf, "  add rax, rdi")?;
            }
            NodeType::Sub => {
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
                self.push_value("rdx")?;
                return Ok(());
            }
            NodeType::Eq | NodeType::Ne | NodeType::Lt | NodeType::Le => {
                writeln!(
                    self.buf,
                    "  cmp rax, rdi\n  {} al\n  {} rax, al",
                    match n.nt {
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
        self.push_value("rax")?;
        Ok(())
    }
}