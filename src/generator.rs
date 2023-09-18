use crate::assembly::Assembly;
use crate::ast::AstBuilder;
use crate::ctype::Type;
use crate::error;
use crate::func::Func;
use crate::global::{GlobalVariable, GlobalVariableData};
use crate::instruction::{
    InstOperand::{self, *},
    InstOperator::{self, *},
    Register::{self, *},
};
use crate::node::{Node, NodeType};
use std::fmt::Display;

pub struct AsmGenerator<'a> {
    error_logger: &'a dyn error::ErrorLogger,
    target_os: Os,
    branch_count: usize,
    loop_stack: Vec<usize>,
    builder: &'a dyn AstBuilder,
    pub assemblies: Vec<Assembly>,
}

#[derive(Clone, Copy)]
pub enum Os {
    Linux,
    MacOS,
}

const ARGS_REG: [Register; 6] = [RDI, RSI, RDX, RCX, R8, R9];

impl<'a> AsmGenerator<'a> {
    pub fn new(
        builder: &'a dyn AstBuilder,
        error_logger: &'a dyn error::ErrorLogger,
        target_os: Os,
    ) -> Self {
        Self {
            error_logger,
            target_os,
            builder,
            branch_count: 0,
            loop_stack: Vec::new(),
            assemblies: Vec::new(),
        }
    }

    pub fn generate_string(&self) -> String {
        self.assemblies
            .iter()
            .map(|ass| {
                if let Os::MacOS = self.target_os {
                    ass.to_string()
                } else {
                    ass.to_string4linux()
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn generate_assemblies(&mut self) -> Vec<Assembly> {
        vec![
            ".intel_syntax noprefix".into(),
            if let Os::MacOS = self.target_os {
                ".section __TEXT,__text,regular,pure_instructions"
            } else {
                ".section .text"
            }
            .into(),
            self.builder
                .functions()
                .iter()
                .filter(|(_, func)| func.body.is_some())
                .map(|(name, func)| {
                    vec![
                        format!(".globl {}", self.with_prefix(name)).into(),
                        format!("{}:", self.with_prefix(name)).into(),
                        Assembly::inst1(PUSH, RBP),
                        Assembly::inst2(MOV, RBP, RSP),
                        Assembly::inst2(SUB, RSP, func.offset_size),
                    ]
                    .into()
                })
                .collect::<Vec<Assembly>>()
                .into(),
        ]
    }

    pub fn gen(&mut self) {
        self.assemblies.push(
            vec![
                ".intel_syntax noprefix".into(),
                if let Os::MacOS = self.target_os {
                    ".section __TEXT,__text,regular,pure_instructions"
                } else {
                    ".section .text"
                }
                .into(),
            ]
            .into(),
        );
        self.builder
            .functions()
            .iter()
            .fold(0, |last_offset, (name, f)| {
                let stack_offset = last_offset + f.offset_size;
                self.gen_func(name, f, stack_offset);
                stack_offset
            });
        self.assemblies.push(
            vec![
                if let Os::MacOS = self.target_os {
                    ".section __DATA,__data"
                } else {
                    ".section .data"
                }
                .into(),
                self.builder
                    .global_variables()
                    .iter()
                    .map(|(s, gv)| self.gen_global_variable(s, gv))
                    .collect::<Vec<Assembly>>()
                    .into(),
                self.gen_string_literals(),
            ]
            .into(),
        )
    }

    pub fn gen_string_literals(&self) -> Assembly {
        if self.builder.string_literals().is_empty() {
            vec![]
        } else {
            vec![
                if let Os::MacOS = self.target_os {
                    ".section __TEXT,__cstring,cstring_literals".into()
                } else {
                    ".section .data".into()
                },
                self.builder
                    .string_literals()
                    .iter()
                    .enumerate()
                    .map(|(i, str)| {
                        vec![
                            format!("L_.str.{}:", i).into(),
                            format!("  .asciz \"{}\"", str).into(),
                        ]
                        .into()
                    })
                    .collect::<Vec<Assembly>>()
                    .into(),
            ]
        }
        .into()
    }

    pub fn gen_global_variable(&self, name: &str, gv: &GlobalVariable) -> Assembly {
        vec![
            format!("{}:", self.with_prefix(name)).into(),
            Self::gen_initializer_element(&gv.ty, gv.data.as_ref()),
        ]
        .into()
    }

    pub fn gen_initializer_element(ty: &Type, data: Option<&GlobalVariableData>) -> Assembly {
        match ty {
            Type::Arr(children_ty, size) => {
                if let Some(GlobalVariableData::Arr(v)) = data {
                    vec![
                        v.iter()
                            .enumerate()
                            .filter(|(i, _)| i < size)
                            .map(|(_, d)| {
                                Self::gen_initializer_element(children_ty.as_ref(), Some(d))
                            })
                            .collect::<Vec<Assembly>>()
                            .into(),
                        format!(
                            "  .zero {}",
                            children_ty.size_of()
                                * v.iter()
                                    .enumerate()
                                    .filter(|(i, _)| i < size)
                                    .fold(*size, |_, _| { size - v.len() })
                        )
                        .into(),
                    ]
                    .into()
                } else {
                    format!("  .zero {}", children_ty.size_of() * *size).into()
                }
            }
            Type::I8 => format!(
                "  .byte {}",
                if let Some(GlobalVariableData::Elem(s)) = data {
                    s
                } else {
                    "0"
                }
            )
            .into(),
            Type::I32 => format!(
                "  .4byte {}",
                if let Some(GlobalVariableData::Elem(s)) = data {
                    s
                } else {
                    "0"
                }
            )
            .into(),
            _ => format!(
                "  .8byte {}",
                if let Some(GlobalVariableData::Elem(s)) = data {
                    s
                } else {
                    "0"
                }
            )
            .into(),
        }
    }

    pub fn gen_func(&mut self, name: &str, func: &Func, offset: usize) {
        if func.body.is_none() {
            return;
        }
        self.push_assembly(format!(".globl {}", self.with_prefix(name)));
        self.push_assembly(format!("{}:", self.with_prefix(name)));

        // prologue
        self.inst1(PUSH, RBP);
        self.inst2(MOV, RBP, RSP);
        self.inst2(SUB, RSP, func.offset_size);
        for (i, arg) in func.args.iter().enumerate() {
            if let NodeType::LocalVar = arg.nt {
                self.inst2(MOV, RAX, RBP);
                self.inst2(SUB, RAX, arg.offset.unwrap());
                self.inst2(MOV, Ptr(RAX, 8), ARGS_REG[i]);
            } else {
                self.error_logger
                    .print_error_position(arg.token.as_ref().unwrap().pos, "ident expected");
            }
        }
        self.gen_with_node(func.body.as_ref().unwrap(), offset);
        self.inst2(MOV, RAX, 0); // default return value
        self.epilogue();
    }

    fn epilogue(&mut self) {
        self.assemblies.push(Assembly::epilogue());
    }

    fn gen_with_node(&mut self, node: &Node, offset: usize) {
        match node.nt {
            NodeType::DefVar => {
                self.gen_with_vec(&node.children, offset);
                return;
            }
            NodeType::CallFunc => {
                self.inst2(MOV, RAX, RSP);
                self.inst2(ADD, RAX, 8);
                self.inst2(MOV, RDI, 16);
                self.inst0(CQO);
                self.inst1(IDIV, RDI);
                self.inst2(SUB, RSP, RDX);
                self.inst1(PUSH, RDX);
                for node in &node.args {
                    self.gen_with_node(node, offset);
                }
                for op in ARGS_REG.iter().take(node.args.len()) {
                    self.inst1(POP, *op);
                }
                self.inst1(CALL, self.with_prefix(&node.global_name));
                self.inst1(POP, RDI);
                self.inst2(ADD, RSP, RDI);
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::If => {
                let branch_num = self.new_branch_num();
                self.gen_with_node(node.cond.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.inst2(CMP, RAX, 0);
                self.inst1(JE, ElseFlag(branch_num));
                self.gen_with_node(node.then.as_ref().unwrap(), offset);
                self.inst1(JMP, EndFlag(branch_num));
                self.push_assembly(format!("{}:", ElseFlag(branch_num)));
                if node.els.is_some() {
                    self.gen_with_node(node.els.as_ref().unwrap(), offset);
                }
                self.push_assembly(format!("{}:", EndFlag(branch_num)));
                return;
            }
            NodeType::While => {
                let branch_num = self.new_branch_num();
                self.loop_stack.push(branch_num);
                self.push_assembly(format!("{}:", BeginFlag(branch_num)));
                self.assemblies.push(Assembly::reset_stack(offset));
                self.gen_with_node(node.cond.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.inst2(CMP, RAX, 0);
                self.inst1(JE, EndFlag(branch_num));
                self.gen_with_node(node.then.as_ref().unwrap(), offset);
                self.inst1(JMP, BeginFlag(branch_num));
                self.push_assembly(format!("{}:", EndFlag(branch_num)));
                self.loop_stack.pop();
                return;
            }
            NodeType::For => {
                let branch_num = self.new_branch_num();
                self.loop_stack.push(branch_num);
                if node.ini.is_some() {
                    self.gen_with_node(node.ini.as_ref().unwrap(), offset);
                    self.inst1(POP, RAX);
                }
                self.push_assembly(format!("{}:", BeginFlag(branch_num)));
                self.assemblies.push(Assembly::reset_stack(offset));
                if node.cond.is_some() {
                    self.gen_with_node(node.cond.as_ref().unwrap(), offset);
                    self.inst1(POP, RAX);
                } else {
                    self.inst2(MOV, RAX, 1);
                }
                self.inst2(CMP, RAX, 0);
                self.inst1(JE, EndFlag(branch_num));
                self.gen_with_node(node.then.as_ref().unwrap(), offset);
                if node.upd.is_some() {
                    self.gen_with_node(node.upd.as_ref().unwrap(), offset);
                    self.inst1(POP, RAX);
                }
                self.inst1(JMP, BeginFlag(branch_num));
                self.push_assembly(format!("{}:", EndFlag(branch_num)));
                self.loop_stack.pop();
                return;
            }
            NodeType::Block => {
                self.gen_with_vec(&node.children, offset);
                return;
            }
            NodeType::Break => {
                if let Some(&branch_num) = self.loop_stack.last() {
                    self.inst1(JMP, EndFlag(branch_num));
                } else {
                    self.error_logger.print_error_position(
                        node.token.as_ref().unwrap().pos,
                        "unexpected break found",
                    );
                }
                return;
            }
            NodeType::Return => {
                self.gen_with_node(node.lhs.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.epilogue();
                return;
            }
            NodeType::Num => {
                self.inst1(PUSH, node.value.unwrap());
                return;
            }
            NodeType::LocalVar | NodeType::GlobalVar => {
                self.gen_addr(node, offset);
                if let Some(Type::Arr(_, _)) = node.resolve_type() {
                    return;
                }
                self.inst1(POP, RAX);
                self.deref_rax(node);
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::Addr => {
                self.gen_addr(node.lhs.as_ref().unwrap(), offset);
                return;
            }
            NodeType::Deref => {
                self.gen_with_node(node.lhs.as_ref().unwrap(), offset);
                if let Some(Type::Arr(..)) = node.lhs.as_ref().unwrap().dest_type() {
                    return;
                }
                self.inst1(POP, RAX);
                self.deref_rax(node);
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::Assign => {
                self.gen_addr(node.lhs.as_ref().unwrap(), offset);
                self.gen_with_node(node.rhs.as_ref().unwrap(), offset);
                self.inst1(POP, RDI);
                self.inst1(POP, RAX);
                self.operation2rdi(node.lhs.as_ref().unwrap().resolve_type(), MOV, RAX);
                self.inst1(PUSH, RDI);
                return;
            }
            NodeType::BitLeft | NodeType::BitRight => {
                self.gen_with_node(node.rhs.as_ref().unwrap(), offset);
                self.gen_with_node(node.lhs.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.inst1(POP, RCX);
                self.inst2(
                    match node.nt {
                        NodeType::BitLeft => SHL,
                        NodeType::BitRight => SAR,
                        _ => {
                            unreachable!()
                        }
                    },
                    RAX,
                    CL,
                );
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::BitNot => {
                self.gen_with_node(node.lhs.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.inst1(NOT, RAX);
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::LogicalAnd => {
                let branch_num = self.new_branch_num();
                self.gen_with_node(node.lhs.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.inst2(CMP, RAX, 0);
                self.inst1(JE, EndFlag(branch_num));
                self.gen_with_node(node.rhs.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.push_assembly(format!("{}:", EndFlag(branch_num)));
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::LogicalOr => {
                let branch_num = self.new_branch_num();
                self.gen_with_node(node.lhs.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.inst2(CMP, RAX, 0);
                self.inst1(JNE, EndFlag(branch_num));
                self.gen_with_node(node.rhs.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.push_assembly(format!("{}:", EndFlag(branch_num)));
                self.inst1(PUSH, RAX);
                return;
            }
            NodeType::SuffixIncr | NodeType::SuffixDecr => {
                self.gen_addr(node.lhs.as_ref().unwrap(), offset);
                self.inst1(POP, RAX);
                self.inst2(MOV, RDI, 1);
                if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                    self.inst2(IMUL, RDI, t.size_of());
                }
                self.inst2(MOV, RDX, RAX);
                self.deref_rax(node.lhs.as_ref().unwrap());
                let op = if let NodeType::SuffixIncr = node.nt {
                    ADD
                } else {
                    SUB
                };
                self.operation2rdi(node.lhs.as_ref().unwrap().resolve_type(), op, RDX);
                self.inst1(PUSH, RAX);
                return;
            }
            _ => {}
        }
        self.gen_with_node(node.rhs.as_ref().unwrap(), offset);
        self.gen_with_node(node.lhs.as_ref().unwrap(), offset);
        self.inst1(POP, RAX);
        self.inst1(POP, RDI);
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
                self.inst2(MOV, RAX, RDX);
            }
            NodeType::Eq | NodeType::Ne | NodeType::Lt | NodeType::Le => {
                self.inst2(CMP, RAX, RDI);
                self.inst1(
                    match node.nt {
                        NodeType::Eq => SETE,
                        NodeType::Ne => SETNE,
                        NodeType::Lt => SETL,
                        NodeType::Le => SETLE,
                        _ => unreachable!(),
                    },
                    AL,
                );
                self.inst2(MOVZX, RAX, AL);
            }
            NodeType::BitAnd => {
                self.inst2(AND, RAX, RDI);
            }
            NodeType::BitXor => {
                self.inst2(XOR, RAX, RDI);
            }
            NodeType::BitOr => {
                self.inst2(OR, RAX, RDI);
            }
            _ => {
                self.error_logger
                    .print_error_position(node.token.as_ref().unwrap().pos, "unexpected node");
            }
        }
        self.inst1(PUSH, RAX);
    }

    fn gen_with_vec(&mut self, v: &Vec<Node>, offset: usize) {
        for node in v {
            self.gen_with_node(node, offset);
            self.inst1(POP, RAX);
            self.assemblies.push(Assembly::reset_stack(offset));
        }
    }

    fn gen_addr(&mut self, node: &Node, offset: usize) {
        match node.nt {
            NodeType::GlobalVar => {
                if !node.dest.is_empty() {
                    self.inst2(LEA, RAX, PtrAdd(RIP, node.dest.clone()));
                } else {
                    self.inst2(LEA, RAX, PtrAdd(RIP, self.with_prefix(&node.global_name)));
                }
                self.inst1(PUSH, RAX);
            }
            NodeType::LocalVar => {
                self.inst2(MOV, RAX, RBP);
                self.inst2(SUB, RAX, node.offset.unwrap());
                self.inst1(PUSH, RAX);
            }
            NodeType::Deref => {
                self.gen_with_node(node.lhs.as_ref().unwrap(), offset);
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn operation2rdi(&mut self, c_type: Option<Type>, operator: InstOperator, from: Register) {
        match c_type {
            Some(Type::I8) => {
                self.inst2(operator, Ptr(from, 1), DIL);
            }
            Some(Type::I32) => {
                self.inst2(operator, Ptr(from, 4), EDI);
            }
            _ => {
                self.inst2(operator, Ptr(from, 8), RDI);
            }
        }
    }

    fn deref_rax(&mut self, node: &Node) {
        match node.resolve_type() {
            Some(Type::I32) => {
                self.inst2(MOVSXD, RAX, Ptr(RAX, 4));
            }
            Some(Type::I8) => {
                self.inst2(MOVSX, RAX, Ptr(RAX, 1));
            }
            _ => {
                self.inst2(MOV, RAX, Ptr(RAX, 8));
            }
        }
    }
    fn inst0(&mut self, operator: InstOperator) {
        self.assemblies.push(Assembly::inst0(operator))
    }
    fn inst1<T1>(&mut self, operator: InstOperator, operand1: T1)
    where
        T1: Into<InstOperand>,
    {
        self.assemblies.push(Assembly::inst1(operator, operand1))
    }
    fn inst2<T1, T2>(&mut self, operator: InstOperator, operand1: T1, operand2: T2)
    where
        T1: Into<InstOperand>,
        T2: Into<InstOperand>,
    {
        self.assemblies
            .push(Assembly::inst2(operator, operand1, operand2))
    }

    fn push_assembly(&mut self, s: impl ToString) {
        self.assemblies.push(Assembly::Other(s.to_string()))
    }

    fn with_prefix<T: Display>(&self, s: T) -> String {
        format!(
            "{}{}",
            if let Os::MacOS = self.target_os {
                "_"
            } else {
                ""
            },
            s
        )
    }

    fn new_branch_num(&mut self) -> usize {
        self.branch_count += 1;
        self.branch_count
    }
}
