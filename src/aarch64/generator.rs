use crate::aarch64::assembly::Assembly;
use crate::aarch64::instruction::{
    InstOperand::*,
    InstOperator::{self, *},
    Register::{self, *},
};
use crate::ast::ProgramAst;
use crate::ctype::Type;
use crate::error;
use crate::func::Func;
use crate::global::{GlobalVariable, GlobalVariableData};
use crate::node::{Node, NodeType};
use crate::Os;
use std::fmt::Display;

pub struct AsmGenerator<'a> {
    error_logger: &'a dyn error::ErrorLogger,
    target_os: Os,
    loop_stack: Vec<usize>,
    pub assemblies: Vec<Assembly>,
}

const ARGS_REG: [Register; 6] = [X13, X12, X11, X10, R8, R9];

impl<'a> crate::generator::Generator for AsmGenerator<'a> {
    fn generate(&mut self, ast: ProgramAst) -> Box<dyn crate::generator::Assembly> {
        self.generate(ast)
    }
}

impl<'a> AsmGenerator<'a> {
    pub fn new(error_logger: &'a dyn error::ErrorLogger, target_os: Os) -> Self {
        Self {
            error_logger,
            target_os,
            loop_stack: Vec::new(),
            assemblies: Vec::new(),
        }
    }
    fn generate(&mut self, ast: ProgramAst) -> Box<dyn crate::generator::Assembly> {
        Box::<Assembly>::new(
            vec![
                ".intel_syntax noprefix".into(),
                if let Os::MacOS = self.target_os {
                    ".section __TEXT,__text,regular,pure_instructions"
                } else {
                    ".section .text"
                }
                .into(),
                ast.functions
                    .iter()
                    .fold((0, vec![]), |(last_offset, mut last_vec), (name, f)| {
                        let stack_offset = last_offset + f.offset_size;
                        last_vec.push(self.gen_func(name, f, stack_offset));
                        (stack_offset, last_vec)
                    })
                    .1
                    .into(),
                if let Os::MacOS = self.target_os {
                    ".section __DATA,__data"
                } else {
                    ".section .data"
                }
                .into(),
                ast.global_variables
                    .iter()
                    .map(|(s, gv)| self.gen_global_variable(s, gv))
                    .collect::<Vec<Assembly>>()
                    .into(),
                self.gen_string_literals(ast.string_literals),
            ]
            .into(),
        )
    }

    pub fn gen_string_literals(&self, string_literals: &Vec<String>) -> Assembly {
        if string_literals.is_empty() {
            vec![]
        } else {
            vec![
                if let Os::MacOS = self.target_os {
                    ".section __TEXT,__cstring,cstring_literals".into()
                } else {
                    ".section .data".into()
                },
                string_literals
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

    pub fn gen_func(&mut self, name: &str, func: &Func, offset: usize) -> Assembly {
        if func.body.is_none() {
            return vec![].into();
        }
        vec![
            format!(".globl {}", self.with_prefix(name)).into(),
            format!("{}:", self.with_prefix(name)).into(),
            // prologue
            Assembly::inst1(PUSH, X14),
            Assembly::inst2(MOV, X14, X15),
            Assembly::inst3(SUB, X15, X15, func.offset_size),
            func.args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    if let NodeType::LocalVar = arg.nt {
                        vec![
                            Assembly::inst2(MOV, X8, X14),
                            Assembly::inst3(SUB, X8, X8, arg.offset.unwrap()),
                            Assembly::inst2(MOV, Ptr(X8, 8), ARGS_REG[i]),
                        ]
                        .into()
                    } else {
                        self.error_logger.print_error_position(
                            arg.token.as_ref().unwrap().pos,
                            "ident expected",
                        );
                        unreachable!()
                    }
                })
                .collect::<Vec<Assembly>>()
                .into(),
            self.gen_with_node(func.body.as_ref().unwrap(), offset),
            Assembly::inst2(MOV, X8, 0), // default return value
            Assembly::epilogue(),
        ]
        .into()
    }

    fn gen_with_node(&mut self, node: &Node, offset: usize) -> Assembly {
        match node.nt {
            NodeType::DefVar => return self.gen_with_vec(&node.children, offset),
            NodeType::CallFunc => {
                return vec![
                    Assembly::inst2(MOV, X8, X15),
                    Assembly::inst3(ADD, X8, X8, 8),
                    Assembly::inst2(MOV, X13, 16),
                    Assembly::inst0(CQO),
                    Assembly::inst1(IDIV, X13),
                    Assembly::inst3(SUB, X15, X15, X11),
                    Assembly::inst1(PUSH, X11),
                    node.args
                        .iter()
                        .map(|node| self.gen_with_node(node, offset))
                        .collect::<Vec<Assembly>>()
                        .into(),
                    ARGS_REG
                        .iter()
                        .take(node.args.len())
                        .map(|op| Assembly::inst1(POP, *op))
                        .collect::<Vec<Assembly>>()
                        .into(),
                    Assembly::inst1(BL, self.with_prefix(&node.global_name)),
                    Assembly::inst1(POP, X13),
                    Assembly::inst3(ADD, X15, X15, X13),
                    Assembly::inst1(PUSH, X8),
                ]
                .into()
            }
            NodeType::If => {
                let branch_num = node.token.as_ref().unwrap().pos;
                return vec![
                    self.gen_with_node(node.cond.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JE, ElseFlag(branch_num)),
                    self.gen_with_node(node.then.as_ref().unwrap(), offset),
                    Assembly::inst1(JMP, EndFlag(branch_num)),
                    format!("{}:", ElseFlag(branch_num)).into(),
                    node.els
                        .as_ref()
                        .map(|node| self.gen_with_node(node, offset))
                        .unwrap_or_else(|| vec![].into()),
                    format!("{}:", EndFlag(branch_num)).into(),
                ]
                .into();
            }
            NodeType::While => {
                let branch_num = node.token.as_ref().unwrap().pos;
                self.loop_stack.push(branch_num);
                let v = vec![
                    format!("{}:", BeginFlag(branch_num)).into(),
                    Assembly::reset_stack(offset),
                    self.gen_with_node(node.cond.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JE, EndFlag(branch_num)),
                    self.gen_with_node(node.then.as_ref().unwrap(), offset),
                    Assembly::inst1(JMP, BeginFlag(branch_num)),
                    format!("{}:", EndFlag(branch_num)).into(),
                ];
                self.loop_stack.pop();
                return v.into();
            }
            NodeType::For => {
                let branch_num = node.token.as_ref().unwrap().pos;
                self.loop_stack.push(branch_num);
                let v = vec![
                    node.ini.as_ref().map_or(Vec::new().into(), |node| {
                        vec![self.gen_with_node(node, offset), Assembly::inst1(POP, X8)].into()
                    }),
                    format!("{}:", BeginFlag(branch_num)).into(),
                    Assembly::reset_stack(offset),
                    node.cond
                        .as_ref()
                        .map_or(Assembly::inst2(MOV, X8, 1), |node| {
                            vec![self.gen_with_node(node, offset), Assembly::inst1(POP, X8)].into()
                        }),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JE, EndFlag(branch_num)),
                    self.gen_with_node(node.then.as_ref().unwrap(), offset),
                    node.upd.as_ref().map_or(Vec::new().into(), |node| {
                        vec![self.gen_with_node(node, offset), Assembly::inst1(POP, X8)].into()
                    }),
                    Assembly::inst1(JMP, BeginFlag(branch_num)),
                    format!("{}:", EndFlag(branch_num)).into(),
                ];
                self.loop_stack.pop();
                return v.into();
            }
            NodeType::Block => {
                return self.gen_with_vec(&node.children, offset);
            }
            NodeType::Break => {
                if let Some(&branch_num) = self.loop_stack.last() {
                    return Assembly::inst1(JMP, EndFlag(branch_num));
                } else {
                    self.error_logger.print_error_position(
                        node.token.as_ref().unwrap().pos,
                        "unexpected break found",
                    );
                }
            }
            NodeType::Return => {
                return vec![
                    self.gen_with_node(node.lhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    Assembly::epilogue(),
                ]
                .into();
            }
            NodeType::Num => {
                return Assembly::inst1(PUSH, node.value.unwrap());
            }
            NodeType::LocalVar | NodeType::GlobalVar => {
                return vec![
                    self.gen_addr(node, offset),
                    if let Some(Type::Arr(_, _)) = node.resolve_type() {
                        vec![].into()
                    } else {
                        vec![
                            Assembly::inst1(POP, X8),
                            self.deref_rax(node),
                            Assembly::inst1(PUSH, X8),
                        ]
                        .into()
                    },
                ]
                .into();
            }
            NodeType::Addr => {
                return self.gen_addr(node.lhs.as_ref().unwrap(), offset);
            }
            NodeType::Deref => {
                return vec![
                    self.gen_with_node(node.lhs.as_ref().unwrap(), offset),
                    if let Some(Type::Arr(..)) = node.lhs.as_ref().unwrap().dest_type() {
                        vec![].into()
                    } else {
                        vec![
                            Assembly::inst1(POP, X8),
                            self.deref_rax(node),
                            Assembly::inst1(PUSH, X8),
                        ]
                        .into()
                    },
                ]
                .into();
            }
            NodeType::Assign => {
                return vec![
                    self.gen_addr(node.lhs.as_ref().unwrap(), offset),
                    self.gen_with_node(node.rhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X13),
                    Assembly::inst1(POP, X8),
                    self.operation2rdi(node.lhs.as_ref().unwrap().resolve_type(), MOV, X8),
                    Assembly::inst1(PUSH, X13),
                ]
                .into();
            }
            NodeType::BitLeft | NodeType::BitRight => {
                return vec![
                    self.gen_with_node(node.rhs.as_ref().unwrap(), offset),
                    self.gen_with_node(node.lhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    Assembly::inst1(POP, X10),
                    Assembly::inst2(
                        match node.nt {
                            NodeType::BitLeft => SHL,
                            NodeType::BitRight => SAR,
                            _ => {
                                unreachable!()
                            }
                        },
                        X8,
                        CL,
                    ),
                    Assembly::inst1(PUSH, X8),
                ]
                .into();
            }
            NodeType::BitNot => {
                return vec![
                    self.gen_with_node(node.lhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    Assembly::inst1(NOT, X8),
                    Assembly::inst1(PUSH, X8),
                ]
                .into();
            }
            NodeType::LogicalAnd => {
                let branch_num = node.token.as_ref().unwrap().pos;
                return vec![
                    self.gen_with_node(node.lhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JE, EndFlag(branch_num)),
                    self.gen_with_node(node.rhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    format!("{}:", EndFlag(branch_num)).into(),
                    Assembly::inst1(PUSH, X8),
                ]
                .into();
            }
            NodeType::LogicalOr => {
                let branch_num = node.token.as_ref().unwrap().pos;
                return vec![
                    self.gen_with_node(node.lhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JNE, EndFlag(branch_num)),
                    self.gen_with_node(node.rhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    format!("{}:", EndFlag(branch_num)).into(),
                    Assembly::inst1(PUSH, X8),
                ]
                .into();
            }
            NodeType::SuffixIncr | NodeType::SuffixDecr => {
                let op = if let NodeType::SuffixIncr = node.nt {
                    ADD
                } else {
                    SUB
                };
                return vec![
                    self.gen_addr(node.lhs.as_ref().unwrap(), offset),
                    Assembly::inst1(POP, X8),
                    Assembly::inst2(MOV, X13, 1),
                    if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                        Assembly::inst2(IMUL, X13, t.size_of())
                    } else {
                        vec![].into()
                    },
                    Assembly::inst2(MOV, X11, X8),
                    self.deref_rax(node.lhs.as_ref().unwrap()),
                    self.operation2rdi(node.lhs.as_ref().unwrap().resolve_type(), op, X11),
                    Assembly::inst1(PUSH, X8),
                ]
                .into();
            }
            _ => {}
        }
        vec![
            self.gen_with_node(node.rhs.as_ref().unwrap(), offset),
            self.gen_with_node(node.lhs.as_ref().unwrap(), offset),
            Assembly::inst1(POP, X8),
            Assembly::inst1(POP, X13),
            match node.nt {
                NodeType::Add => vec![
                    if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                        Assembly::inst2(IMUL, X13, t.size_of())
                    } else {
                        vec![].into()
                    },
                    Assembly::inst3(ADD, X8, X8, X13),
                ]
                .into(),
                NodeType::Sub => vec![
                    if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                        Assembly::inst2(IMUL, X13, t.size_of())
                    } else {
                        vec![].into()
                    },
                    Assembly::inst3(SUB, X8, X8, X13),
                ]
                .into(),
                NodeType::Mul => Assembly::inst2(IMUL, X8, X13),
                NodeType::Div => vec![Assembly::inst0(CQO), Assembly::inst1(IDIV, X13)].into(),
                NodeType::Mod => vec![
                    Assembly::inst0(CQO),
                    Assembly::inst1(IDIV, X13),
                    Assembly::inst2(MOV, X8, X11),
                ]
                .into(),
                NodeType::Eq | NodeType::Ne | NodeType::Lt | NodeType::Le => vec![
                    Assembly::inst2(CMP, X8, X13),
                    Assembly::inst1(
                        match node.nt {
                            NodeType::Eq => SETE,
                            NodeType::Ne => SETNE,
                            NodeType::Lt => SETL,
                            NodeType::Le => SETLE,
                            _ => unreachable!(),
                        },
                        AL,
                    ),
                    Assembly::inst2(MOVZX, X8, AL),
                ]
                .into(),
                NodeType::BitAnd => Assembly::inst2(AND, X8, X13),
                NodeType::BitXor => Assembly::inst2(XOR, X8, X13),
                NodeType::BitOr => Assembly::inst2(OR, X8, X13),
                _ => {
                    self.error_logger
                        .print_error_position(node.token.as_ref().unwrap().pos, "unexpected node");
                    unreachable!();
                }
            },
            Assembly::inst1(PUSH, X8),
        ]
        .into()
    }

    fn gen_with_vec(&mut self, v: &[Node], offset: usize) -> Assembly {
        v.iter()
            .map(|node| {
                vec![
                    self.gen_with_node(node, offset),
                    Assembly::inst1(POP, X8),
                    Assembly::reset_stack(offset),
                ]
                .into()
            })
            .collect::<Vec<Assembly>>()
            .into()
    }

    fn gen_addr(&mut self, node: &Node, offset: usize) -> Assembly {
        match node.nt {
            NodeType::GlobalVar => vec![
                if !node.dest.is_empty() {
                    Assembly::inst2(LEA, X8, PtrAdd(RIP, node.dest.clone()))
                } else {
                    Assembly::inst2(LEA, X8, PtrAdd(RIP, self.with_prefix(&node.global_name)))
                },
                Assembly::inst1(PUSH, X8),
            ]
            .into(),
            NodeType::LocalVar => vec![
                Assembly::inst2(MOV, X8, X14),
                Assembly::inst3(SUB, X8, X8, node.offset.unwrap()),
                Assembly::inst1(PUSH, X8),
            ]
            .into(),
            NodeType::Deref => self.gen_with_node(node.lhs.as_ref().unwrap(), offset),
            _ => {
                unreachable!();
            }
        }
    }

    fn operation2rdi(
        &mut self,
        c_type: Option<Type>,
        operator: InstOperator,
        from: Register,
    ) -> Assembly {
        match c_type {
            Some(Type::I8) => Assembly::inst2(operator, Ptr(from, 1), X17),
            Some(Type::I32) => Assembly::inst2(operator, Ptr(from, 4), X16),
            _ => Assembly::inst2(operator, Ptr(from, 8), X13),
        }
    }

    fn deref_rax(&mut self, node: &Node) -> Assembly {
        match node.resolve_type() {
            Some(Type::I32) => Assembly::inst2(MOVSXD, X8, Ptr(X8, 4)),
            Some(Type::I8) => Assembly::inst2(MOVSX, X8, Ptr(X8, 1)),
            _ => Assembly::inst2(MOV, X8, Ptr(X8, 8)),
        }
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
}
