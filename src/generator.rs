use std::io::Write;
use crate::node::{Node, NodeType};
use crate::error::{error, error_at};
use std::collections::VecDeque;
use crate::ast::{AstBuilder};
use crate::ctype::Type;
use crate::func::Func;
use crate::instruction::{Instruction, InstOperator, InstOperand};
use crate::instruction::{InstOperator::*, Register::*};
use std::fmt::Display;
use crate::global::{GlobalVariable, GlobalVariableData};

pub struct AsmGenerator<'a> {
    code: &'a str,
    pub buf: Vec<u8>,
    target_os: Os,
    branch_count: usize,
    loop_stack: VecDeque<usize>,
    builder: &'a AstBuilder<'a>,
    current_stack_size: usize,
    pub instructions: Vec<Instruction>,
}

pub enum Os {
    Linux,
    MacOS,
}

const ARGS_REG: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl<'a> AsmGenerator<'a> {
    pub fn new(builder: &'a AstBuilder<'a>, code: &'a str, target_os: Os) -> Self {
        Self {
            code,
            buf: Vec::new(),
            target_os,
            builder,
            branch_count: 0,
            loop_stack: VecDeque::new(),
            current_stack_size: 0,
            instructions: Vec::new(),
        }
    }

    pub fn gen(&mut self) {
        self.writeln(".intel_syntax noprefix");
        if let Os::MacOS = self.target_os {
            self.writeln(".section __TEXT,__text,regular,pure_instructions");
        }
        for (s, f) in &self.builder.global_functions {
            self.gen_func(s, f);
        }
        if self.builder.global_variables.len() != 0 {
            if let Os::MacOS = self.target_os {
                self.writeln(".section __DATA,__data");
            } else {
                self.writeln(".section .data");
            }
        }
        for (s, gv) in &self.builder.global_variables {
            self.gen_global_variable(s, gv);
        }
        self.gen_string_literals();
    }

    pub fn gen_string_literals(&mut self) {
        if self.builder.string_literals.len() != 0 {
            if let Os::MacOS = self.target_os {
                self.writeln(".section __TEXT,__cstring,cstring_literals");
            } else {
                self.writeln(".section .data");
            }
            for (i, str) in self.builder.string_literals.iter().enumerate() {
                self.writeln(format!("L_.str.{}:", i));
                self.writeln(format!("  .asciz \"{}\"", str));
            }
        }
    }

    pub fn gen_global_variable(&mut self, name: &str, gv: &GlobalVariable) {
        self.writeln(format!("{}:", self.with_prefix(name)));
        self.gen_initializer_element(&gv.ty, gv.data.as_ref())
    }

    pub fn gen_initializer_element(&mut self, ty: &Type, data: Option<&GlobalVariableData>) {
        match ty {
            Type::Arr(children_ty, size) => {
                let mut rest_count = *size;
                if let Some(GlobalVariableData::Arr(v)) = data {
                    for (i, d) in v.iter().enumerate() {
                        if i >= *size {
                            break;
                        }
                        self.gen_initializer_element(children_ty.as_ref(), Some(d));
                    }
                    rest_count -= v.len();
                }
                self.writeln(format!("  .zero {}", children_ty.size_of() * rest_count));
            }
            Type::Char => {
                self.writeln(format!(
                    "  .byte {}",
                    if let Some(GlobalVariableData::Elem(s)) = data { s } else { "0" }
                ));
            }
            Type::Int => {
                self.writeln(format!(
                    "  .4byte {}",
                    if let Some(GlobalVariableData::Elem(s)) = data { s } else { "0" }
                ));
            }
            _ => {
                self.writeln(format!(
                    "  .8byte {}",
                    if let Some(GlobalVariableData::Elem(s)) = data { s } else { "0" }
                ));
            }
        }
    }

    pub fn gen_func(&mut self, name: &str, func: &Func) {
        if let None = func.body {
            return;
        }
        self.writeln(format!(".globl {}", self.with_prefix(name)));
        self.writeln(format!("{}:", self.with_prefix(name)));

        // prologue
        self.inst1(PUSH, RBP);
        self.inst2(MOV, RBP, RSP);
        self.inst2(SUB, RSP, func.offset_size);
        for (i, arg) in func.args.iter().enumerate() {
            if let NodeType::LVar = arg.nt {
                self.inst2(MOV, RAX, RBP);
                self.inst2(SUB, RAX, arg.offset.unwrap());
                self.inst2(MOV, "[rax]", ARGS_REG[i]);
            } else {
                error_at(self.code, arg.token.as_ref().unwrap().pos, "ident expected");
            }
        }
        self.current_stack_size = func.offset_size;
        self.gen_with_node(func.body.as_ref().unwrap());
        self.epilogue();
    }

    fn epilogue(&mut self) {
        self.inst2(MOV, RSP, RBP);
        self.inst1(POP, RBP);
        self.inst0(RET);
    }

    fn gen_with_node(&mut self, node: &Node) {
        match node.nt {
            NodeType::DefVar => {
                self.gen_with_vec(&node.children);
                return;
            }
            NodeType::Cf => {
                self.inst2(MOV, RAX, RSP);
                self.inst2(ADD, RAX, 8);
                self.inst2(MOV, RDI, 16);
                self.inst0(CQO);
                self.inst1(IDIV, RDI);
                self.inst2(SUB, RSP, RDX);
                self.inst1(PUSH, RDX);
                for node in &node.args {
                    self.gen_with_node(node);
                }
                for i in 0..node.args.len() {
                    self.inst1(POP, ARGS_REG[i]);
                }
                self.inst1(CALL, self.with_prefix(&node.global_name));
                self.inst1(POP, RDI);
                self.inst2(ADD, RSP, RDI);
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::If => {
                let branch_num = self.new_branch_num();
                self.gen_with_node(node.cond.as_ref().unwrap());
                self.inst1(POP, RAX);
                self.inst2(CMP, RAX, 0);
                self.inst1(JE, else_flag(branch_num));
                self.gen_with_node(node.then.as_ref().unwrap());
                self.inst1(JMP, end_flag(branch_num));
                self.writeln(format!(".Lelse{}:", branch_num));
                if let Some(_) = node.els {
                    self.gen_with_node(node.els.as_ref().unwrap());
                }
                self.writeln(format!(".Lend{}:", branch_num));
                return;
            }
            NodeType::Whl => {
                let branch_num = self.new_branch_num();
                self.loop_stack.push_back(branch_num);
                self.writeln(format!(".Lbegin{}:", branch_num));
                self.reset_stack();
                self.gen_with_node(node.cond.as_ref().unwrap());
                self.inst1(POP, RAX);
                self.inst2(CMP, RAX, 0);
                self.inst1(JE, end_flag(branch_num));
                self.gen_with_node(node.then.as_ref().unwrap());
                self.inst1(JMP, begin_flag(branch_num));
                self.writeln(format!(".Lend{}:", branch_num));
                self.loop_stack.pop_back();
                return;
            }
            NodeType::For => {
                let branch_num = self.new_branch_num();
                self.loop_stack.push_back(branch_num);
                if let Some(_) = node.ini {
                    self.gen_with_node(node.ini.as_ref().unwrap());
                    self.inst1(POP, RAX);
                }
                self.writeln(format!(".Lbegin{}:", branch_num));
                self.reset_stack();
                if let Some(_) = node.cond {
                    self.gen_with_node(node.cond.as_ref().unwrap());
                    self.inst1(POP, RAX);
                } else {
                    self.inst2(MOV, RAX, 1);
                }
                self.inst2(CMP, RAX, 0);
                self.inst1(JE, end_flag(branch_num));
                self.gen_with_node(node.then.as_ref().unwrap());
                if let Some(_) = node.upd {
                    self.gen_with_node(node.upd.as_ref().unwrap());
                    self.inst1(POP, RAX);
                }
                self.inst1(JMP, begin_flag(branch_num));
                self.writeln(format!(".Lend{}:", branch_num));
                self.loop_stack.pop_back();
                return;
            }
            NodeType::Block => {
                self.gen_with_vec(&node.children);
                return;
            }
            NodeType::Brk => {
                if let Some(&branch_num) = self.loop_stack.back() {
                    self.inst1(JMP, end_flag(branch_num.clone()));
                } else {
                    error_at(self.code, node.token.as_ref().unwrap().pos, "unexpected break found");
                }
                return;
            }
            NodeType::Ret => {
                self.gen_with_node(node.lhs.as_ref().unwrap());
                self.inst1(POP, RAX);
                self.epilogue();
                return;
            }
            NodeType::Num => {
                self.inst1(PUSH, node.value.unwrap());
                return;
            }
            NodeType::LVar | NodeType::GVar => {
                self.gen_addr(node);
                if let Some(Type::Arr(_, _)) = node.resolve_type() {
                    return;
                }
                self.inst1(POP, RAX);
                self.deref_rax(node);
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::Addr => {
                self.gen_addr(node.lhs.as_ref().unwrap());
                return;
            }
            NodeType::Deref => {
                self.gen_with_node(node.lhs.as_ref().unwrap());
                if let Some(Type::Arr(..)) = node.lhs.as_ref().unwrap().dest_type() {
                    return;
                }
                self.inst1(POP, RAX);
                self.deref_rax(node);
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::Asg => {
                self.gen_addr(node.lhs.as_ref().unwrap());
                self.gen_with_node(node.rhs.as_ref().unwrap());
                self.inst1(POP, RDI);
                self.inst1(POP, RAX);
                match node.lhs.as_ref().unwrap().resolve_type() {
                    Some(Type::Int) => {
                        self.inst2(MOV, ptr_with_size(RAX, 4), EDI);
                    }
                    Some(Type::Char) => {
                        self.inst2(MOV, ptr_with_size(RAX, 1), DIL);
                    }
                    _ => {
                        self.inst2(MOV, ptr_with_size(RAX, 8), RDI);
                    }
                }
                self.inst1(PUSH, RDI);
                return;
            }
            _ => {}
        }
        self.gen_with_node(node.lhs.as_ref().unwrap());
        self.gen_with_node(node.rhs.as_ref().unwrap());
        self.inst1(POP, RDI);
        self.inst1(POP, RAX);
        match node.nt {
            NodeType::Add => {
                if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                    self.inst2(IMUL, RDI, t.size_of());
                }
                self.inst2(ADD, RAX, RDI);
            }
            NodeType::Sub => {
                if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                    self.inst2(IMUL, RDI, t.size_of());
                }
                self.inst2(SUB, RAX, RDI);
            }
            NodeType::Mul => {
                self.inst2(IMUL, RAX, RDI);
            }
            NodeType::Div => {
                self.inst0(CQO);
                self.inst1(IDIV, RDI);
            }
            NodeType::Mod => {
                self.inst0(CQO);
                self.inst1(IDIV, RDI);
                self.inst1(PUSH, RDX);
                return;
            }
            NodeType::Eq | NodeType::Ne | NodeType::Lt | NodeType::Le => {
                self.inst2(CMP, RAX, RDI);
                self.inst1(match node.nt {
                    NodeType::Eq => SETE,
                    NodeType::Ne => SETNE,
                    NodeType::Lt => SETL,
                    NodeType::Le => SETLE,
                    _ => unreachable!()
                }, AL);
                self.inst2(MOVZX, RAX, AL);
            }
            _ => {
                error("unexpected node");
            }
        }
        self.inst1(PUSH, RAX);
    }

    fn gen_with_vec(&mut self, v: &Vec<Node>) {
        for node in v {
            self.gen_with_node(node);
            self.inst1(POP, RAX);
            self.reset_stack();
        }
    }

    fn gen_addr(&mut self, node: &Node) {
        match node.nt {
            NodeType::GVar => {
                if node.dest != "" {
                    self.inst2(LEA, RAX, ptr_with_offset(RIP, &node.dest));
                } else {
                    self.inst2(LEA, RAX,
                               ptr_with_offset(RIP, self.with_prefix(&node.global_name)));
                }
                self.inst1(PUSH, RAX);
            }
            NodeType::LVar => {
                self.inst2(MOV, RAX, RBP);
                self.inst2(SUB, RAX, node.offset.unwrap());
                self.inst1(PUSH, RAX);
            }
            NodeType::Deref => {
                self.gen_with_node(node.lhs.as_ref().unwrap());
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn deref_rax(&mut self, node: &Node) {
        match node.resolve_type() {
            Some(Type::Int) => {
                self.inst2(MOVSXD, RAX, ptr_with_size(RAX, 4));
            }
            Some(Type::Char) => {
                self.inst2(MOVSX, RAX, ptr_with_size(RAX, 1));
            }
            _ => {
                self.inst2(MOV, RAX, ptr_with_size(RAX, 8));
            }
        }
    }

    fn inst0(&mut self, operator: InstOperator) {
        writeln!(
            self.buf, "  {}",
            if let Os::MacOS = self.target_os { operator.to_string() } else { operator.to_string_for_linux() }
        ).unwrap();
        self.instructions.push(
            Instruction { operator, operand1: None, operand2: None }
        )
    }
    fn inst1<T1>(&mut self, operator: InstOperator, operand1: T1) where
        T1: Into<InstOperand> + std::fmt::Display {
        writeln!(
            self.buf,
            "  {} {}",
            if let Os::MacOS = self.target_os { operator.to_string() } else { operator.to_string_for_linux() },
            operand1
        ).unwrap();
        self.instructions.push(
            Instruction { operator, operand1: Some(operand1.into()), operand2: None }
        )
    }
    fn inst2<T1, T2>(&mut self, operator: InstOperator, operand1: T1, operand2: T2) where
        T1: Into<InstOperand> + std::fmt::Display, T2: Into<InstOperand> + std::fmt::Display {
        writeln!(
            self.buf,
            "  {} {}, {}",
            if let Os::MacOS = self.target_os { operator.to_string() } else { operator.to_string_for_linux() },
            operand1,
            operand2
        ).unwrap();
        self.instructions.push(
            Instruction { operator, operand1: Some(operand1.into()), operand2: Some(operand2.into()) }
        )
    }

    fn writeln(&mut self, s: impl Display) {
        writeln!(self.buf, "{}", s).unwrap();
    }

    fn with_prefix<T: Display>(&self, s: T) -> String {
        format!("{}{}", if let Os::MacOS = self.target_os { "_" } else { "" }, s)
    }

    fn new_branch_num(&mut self) -> usize {
        self.branch_count += 1;
        self.branch_count
    }

    fn reset_stack(&mut self) {
        self.inst2(MOV, RSP, RBP);
        self.inst2(SUB, RSP, self.current_stack_size);
    }
}

fn ptr_with_offset(a: impl Display, b: impl Display) -> String {
    format!("[{} + {}]", a, b)
}

fn else_flag(branch_number: usize) -> String {
    format!(".Lelse{}", branch_number)
}

fn end_flag(branch_number: usize) -> String {
    format!(".Lend{}", branch_number)
}

fn begin_flag(branch_number: usize) -> String {
    format!(".Lbegin{}", branch_number)
}

fn ptr_with_size(ptr: impl Display, size: usize) -> String {
    match size {
        4 => format!("dword ptr[{}]", ptr),
        1 => format!("byte ptr[{}]", ptr),
        8 => format!("qword ptr[{}]", ptr),
        _ => unreachable!()
    }
}