use super::assembly::Assembly;
use super::instruction::{
    InstOperand::{self, *},
    InstOperator::{self, *},
    Register::{self, *},
};
use crate::ast::{Identifier, ProgramAst, reserved_functions};
use crate::ctype::Type;
use crate::error;
use crate::func::Func;
use crate::generator::Os;
use crate::global::{GlobalVariable, GlobalVariableData};
use crate::node::{Node, NodeType};
use std::fmt::Display;

pub struct AsmGenerator<'a> {
    error_logger: &'a dyn error::ErrorLogger,
    target_os: Os,
}

const ARGS_REG: [Register; 8] = [X0, X1, X2, X3, X4, X5, X6, X7];

impl crate::generator::Generator for AsmGenerator<'_> {
    fn generate(&self, ast: ProgramAst) -> Box<dyn crate::generator::Assembly> {
        self.generate(ast)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    offset: usize,
    breakable_branch_num: usize,
    args_offset: usize,
}

impl<'a> AsmGenerator<'a> {
    pub fn new(error_logger: &'a dyn error::ErrorLogger, target_os: Os) -> Self {
        Self {
            error_logger,
            target_os,
        }
    }
    fn generate(&self, ast: ProgramAst) -> Box<dyn crate::generator::Assembly> {
        Box::<Assembly>::new(
            vec![
                if let Os::MacOS = self.target_os {
                    ".section __TEXT,__text,regular,pure_instructions"
                } else {
                    ".section .text"
                }
                .into(),
                ast.functions
                    .iter()
                    .map(|(name, f)| {
                        let func_offset_with_alignment = f.offset_size.div_ceil(16) * 16;
                        self.gen_func(name, f, func_offset_with_alignment)
                    })
                    .collect::<Vec<Assembly>>()
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

    fn gen_string_literals(&self, string_literals: &Vec<String>) -> Assembly {
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

    fn gen_global_variable(&self, name: &str, gv: &GlobalVariable) -> Assembly {
        vec![
            format!("{}:", self.with_prefix(name)).into(),
            Self::gen_initializer_element(&gv.ty, gv.data.as_ref()),
        ]
        .into()
    }

    fn gen_initializer_element(ty: &Type, data: Option<&GlobalVariableData>) -> Assembly {
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

    fn gen_func(&self, name: &str, func: &Func, offset: usize) -> Assembly {
        if func.body.is_none() {
            return vec![].into();
        }
        vec![
            format!("	.globl	{}", self.with_prefix(name)).into(),
            "	.p2align	2".into(),
            format!("{}:", self.with_prefix(name)).into(),
            // prologue
            Assembly::inst3(SUB, SP, SP, offset),
            Assembly::inst3(STP, X29, X30, PtrAdd(SP, "#16".to_string())),
            Assembly::inst2(MOV, X9, SP),
            func.args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    if let NodeType::LocalVar = arg.nt {
                        Assembly::inst2(
                            STR,
                            ARGS_REG[i],
                            PtrAdd(SP, format!("#{}", arg.offset.unwrap())),
                        )
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
            self.gen_node(
                func.body.as_ref().unwrap(),
                Options {
                    offset,
                    breakable_branch_num: 0,
                    args_offset: 0,
                },
            ),
            Assembly::inst2(MOV, X0, 0), // default return value
            Self::epilogue(offset),
        ]
        .into()
    }
    fn push<T1>(operand: T1) -> Assembly
    where
        T1: Into<InstOperand>,
    {
        vec![
            Assembly::inst3(SUB, X9, X9, 8),
            Assembly::inst2(MOV, X8, operand),
            Assembly::inst2(STR, X8, Ptr(X9, 8)),
        ]
        .into()
    }
    fn pop<T1>(operand: T1) -> Assembly
    where
        T1: Into<InstOperand>,
    {
        vec![
            Assembly::inst2(LDR, operand, Ptr(X9, 8)),
            Assembly::inst3(ADD, X9, X9, 8),
        ]
        .into()
    }

    fn reset_stack(stack_size: usize) -> Assembly {
        vec![Assembly::inst3(ADD, X9, X9, stack_size)].into()
    }
    fn epilogue(offset: usize) -> Assembly {
        Assembly::Group(vec![
            Assembly::inst3(LDP, X29, X30, PtrAdd(SP, "#16".to_string())),
            Assembly::inst3(ADD, SP, SP, offset),
            Assembly::inst0(RET),
        ])
    }

    fn gen_node(&self, node: &Node, options: Options) -> Assembly {
        match node.nt {
            NodeType::DefVar => return self.gen_statements(&node.children, options),
            NodeType::CallFunc => {
                let fixed_args_len = reserved_functions()
                    .get(node.global_name.as_str())
                    .map(|v| {
                        if let Os::Linux = self.target_os {
                            return node.args.len().min(ARGS_REG.len());
                        }
                        let Identifier::Static(Type::Func(args, _)) = v else {
                            unreachable!()
                        };
                        args.len()
                    })
                    .unwrap_or(node.args.len());
                let variadic_args_len = node.args.len() - fixed_args_len;
                let args_offset = (options.args_offset + variadic_args_len * 8) % 16;
                let options = Options {
                    args_offset,
                    ..options
                };
                return vec![
                    Assembly::inst3(SUB, X9, X9, args_offset), // if the length of variadic args is odd, we need to align the stack
                    node.args
                        .iter()
                        .map(|node| self.gen_node(node, options))
                        .collect::<Vec<Assembly>>()
                        .into(),
                    ARGS_REG
                        .iter()
                        .take(fixed_args_len)
                        .map(|op| Self::pop(*op))
                        .collect::<Vec<Assembly>>()
                        .into(),
                    Assembly::inst2(MOV, SP, X9),
                    Assembly::inst1(BL, self.with_prefix(&node.global_name)),
                    Assembly::inst3(ADD, SP, SP, variadic_args_len * 8 + args_offset), // restore SP to the original value before calling the function
                    Assembly::inst2(MOV, X9, SP),
                    Self::push(X0), // push the return value
                ]
                .into();
            }
            NodeType::If => {
                let branch_num = node.token.as_ref().unwrap().pos;
                return vec![
                    self.gen_node(node.cond.as_ref().unwrap(), options),
                    Self::pop(X8),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JE, ElseFlag(branch_num)),
                    self.gen_node(node.then.as_ref().unwrap(), options),
                    Assembly::inst1(JMP, EndFlag(branch_num)),
                    format!("{}:", ElseFlag(branch_num)).into(),
                    node.els
                        .as_ref()
                        .map(|node| self.gen_node(node, options))
                        .unwrap_or_else(|| vec![].into()),
                    format!("{}:", EndFlag(branch_num)).into(),
                ]
                .into();
            }
            NodeType::While => {
                let branch_num = node.token.as_ref().unwrap().pos;
                let options = Options {
                    breakable_branch_num: branch_num,
                    ..options
                };
                let v = vec![
                    format!("{}:", BeginFlag(branch_num)).into(),
                    Self::reset_stack(options.offset),
                    self.gen_node(node.cond.as_ref().unwrap(), options),
                    Self::pop(X8),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JE, EndFlag(branch_num)),
                    self.gen_node(node.then.as_ref().unwrap(), options),
                    Assembly::inst1(JMP, BeginFlag(branch_num)),
                    format!("{}:", EndFlag(branch_num)).into(),
                ];
                return v.into();
            }
            NodeType::For => {
                let branch_num = node.token.as_ref().unwrap().pos;
                let options = Options {
                    breakable_branch_num: branch_num,
                    ..options
                };
                let v = vec![
                    node.ini.as_ref().map_or(Vec::new().into(), |node| {
                        vec![self.gen_node(node, options), Self::pop(X8)].into()
                    }),
                    format!("{}:", BeginFlag(branch_num)).into(),
                    Self::reset_stack(options.offset),
                    node.cond
                        .as_ref()
                        .map_or(Assembly::inst2(MOV, X8, 1), |node| {
                            vec![self.gen_node(node, options), Self::pop(X8)].into()
                        }),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JE, EndFlag(branch_num)),
                    self.gen_node(node.then.as_ref().unwrap(), options),
                    node.upd.as_ref().map_or(Vec::new().into(), |node| {
                        vec![self.gen_node(node, options), Self::pop(X8)].into()
                    }),
                    Assembly::inst1(JMP, BeginFlag(branch_num)),
                    format!("{}:", EndFlag(branch_num)).into(),
                ];
                return v.into();
            }
            NodeType::Block => {
                return self.gen_statements(&node.children, options);
            }
            NodeType::Break => {
                if options.breakable_branch_num != 0 {
                    return Assembly::inst1(JMP, EndFlag(options.breakable_branch_num));
                } else {
                    self.error_logger.print_error_position(
                        node.token.as_ref().unwrap().pos,
                        "unexpected break found",
                    );
                }
            }
            NodeType::Return => {
                return vec![
                    self.gen_node(node.lhs.as_ref().unwrap(), options),
                    Self::pop(X0),
                    Self::epilogue(options.offset),
                ]
                .into();
            }
            NodeType::Num => {
                return Self::push(node.value.unwrap());
            }
            NodeType::LocalVar | NodeType::GlobalVar => {
                return vec![
                    self.gen_addr(node, options),
                    if let Some(Type::Arr(_, _)) = node.resolve_type() {
                        vec![].into()
                    } else {
                        vec![Self::pop(X8), self.deref(node), Self::push(X8)].into()
                    },
                ]
                .into();
            }
            NodeType::Addr => {
                return self.gen_addr(node.lhs.as_ref().unwrap(), options);
            }
            NodeType::Deref => {
                return vec![
                    self.gen_node(node.lhs.as_ref().unwrap(), options),
                    if let Some(Type::Arr(..)) = node.lhs.as_ref().unwrap().dest_type() {
                        vec![].into()
                    } else {
                        vec![Self::pop(X8), self.deref(node), Self::push(X8)].into()
                    },
                ]
                .into();
            }
            NodeType::Assign => {
                return vec![
                    self.gen_addr(node.lhs.as_ref().unwrap(), options),
                    self.gen_node(node.rhs.as_ref().unwrap(), options),
                    Self::pop(X13),
                    Self::pop(X8),
                    Assembly::inst2(LDR, X13, Ptr(X8, 8)),
                    //self.operation2rdi(node.lhs.as_ref().unwrap().resolve_type(), MOV, X8),
                    Self::push(X13),
                ]
                .into();
            }
            NodeType::BitLeft | NodeType::BitRight => {
                return vec![
                    self.gen_node(node.rhs.as_ref().unwrap(), options),
                    self.gen_node(node.lhs.as_ref().unwrap(), options),
                    Self::pop(X8),
                    Self::pop(X10),
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
                    Self::push(X8),
                ]
                .into();
            }
            NodeType::BitNot => {
                return vec![
                    self.gen_node(node.lhs.as_ref().unwrap(), options),
                    Self::pop(X8),
                    Assembly::inst1(NOT, X8),
                    Self::push(X8),
                ]
                .into();
            }
            NodeType::LogicalAnd => {
                let branch_num = node.token.as_ref().unwrap().pos;
                return vec![
                    self.gen_node(node.lhs.as_ref().unwrap(), options),
                    Self::pop(X8),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JE, EndFlag(branch_num)),
                    self.gen_node(node.rhs.as_ref().unwrap(), options),
                    Self::pop(X8),
                    format!("{}:", EndFlag(branch_num)).into(),
                    Self::push(X8),
                ]
                .into();
            }
            NodeType::LogicalOr => {
                let branch_num = node.token.as_ref().unwrap().pos;
                return vec![
                    self.gen_node(node.lhs.as_ref().unwrap(), options),
                    Self::pop(X8),
                    Assembly::inst2(CMP, X8, 0),
                    Assembly::inst1(JNE, EndFlag(branch_num)),
                    self.gen_node(node.rhs.as_ref().unwrap(), options),
                    Self::pop(X8),
                    format!("{}:", EndFlag(branch_num)).into(),
                    Self::push(X8),
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
                    self.gen_addr(node.lhs.as_ref().unwrap(), options),
                    Self::pop(X8),
                    Assembly::inst2(MOV, X13, 1),
                    if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                        Assembly::inst2(MUL, X13, t.size_of())
                    } else {
                        vec![].into()
                    },
                    Assembly::inst2(MOV, X11, X8),
                    self.deref(node.lhs.as_ref().unwrap()),
                    self.operation2rdi(node.lhs.as_ref().unwrap().resolve_type(), op, X11),
                    Self::push(X8),
                ]
                .into();
            }
            _ => {}
        }
        vec![
            self.gen_node(node.rhs.as_ref().unwrap(), options),
            self.gen_node(node.lhs.as_ref().unwrap(), options),
            Self::pop(X8),
            Self::pop(X13),
            match node.nt {
                NodeType::Add => vec![
                    if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                        Assembly::inst2(MUL, X13, t.size_of())
                    } else {
                        vec![].into()
                    },
                    Assembly::inst3(ADD, X8, X8, X13),
                ]
                .into(),
                NodeType::Sub => vec![
                    if let Some(t) = node.lhs.as_ref().unwrap().dest_type() {
                        Assembly::inst2(MUL, X13, t.size_of())
                    } else {
                        vec![].into()
                    },
                    Assembly::inst3(SUB, X8, X8, X13),
                ]
                .into(),
                NodeType::Mul => Assembly::inst3(MUL, X8, X8, X13),
                NodeType::Div => Assembly::inst3(SDIV, X8, X8, X13),
                NodeType::Mod => vec![
                    Assembly::inst3(SDIV, X10, X8, X13),
                    Assembly::inst4(MSUB, X8, X10, X13, X8),
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
            Self::push(X8),
        ]
        .into()
    }

    fn gen_statements(&self, v: &[Node], options: Options) -> Assembly {
        v.iter()
            .map(|node| {
                vec![
                    self.gen_node(node, options),
                    Self::pop(X8),
                    // Self::reset_stack(offset),
                ]
                .into()
            })
            .collect::<Vec<Assembly>>()
            .into()
    }

    fn gen_addr(&self, node: &Node, options: Options) -> Assembly {
        match node.nt {
            NodeType::GlobalVar => vec![
                if !node.dest.is_empty() {
                    vec![
                        Assembly::inst2(
                            ADRP,
                            X8,
                            match self.target_os {
                                Os::MacOS => node.dest.clone().replace('@', "L_") + "@PAGE",
                                Os::Linux => node.dest.clone().replace('@', "L_"),
                            },
                        ),
                        Assembly::inst3(
                            ADD,
                            X8,
                            X8,
                            match self.target_os {
                                Os::MacOS => node.dest.clone().replace('@', "L_") + "@PAGEOFF",
                                Os::Linux => {
                                    ":lo12:".to_string() + &node.dest.clone().replace('@', "L_")
                                }
                            },
                        ),
                    ]
                    .into()
                } else {
                    vec![
                        Assembly::inst2(
                            ADRP,
                            X8,
                            match self.target_os {
                                Os::MacOS => self.with_prefix(&node.global_name) + "@PAGE",
                                Os::Linux => self.with_prefix(&node.global_name),
                            },
                        ),
                        Assembly::inst3(
                            ADD,
                            X8,
                            X8,
                            match self.target_os {
                                Os::MacOS => self.with_prefix(&node.global_name) + "@PAGEOFF",
                                Os::Linux => {
                                    ":lo12:".to_string()
                                        + &self.with_prefix(&node.global_name).replace('@', "L_")
                                }
                            },
                        ),
                    ]
                    .into()
                },
                Self::push(X8),
            ]
            .into(),
            NodeType::LocalVar => vec![
                Assembly::inst3(ADD, X8, SP, node.offset.unwrap()),
                Self::push(X8),
            ]
            .into(),
            NodeType::Deref => self.gen_node(node.lhs.as_ref().unwrap(), options),
            _ => {
                unreachable!();
            }
        }
    }

    fn operation2rdi(
        &self,
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

    fn deref(&self, node: &Node) -> Assembly {
        match node.resolve_type() {
            Some(Type::I8) => Assembly::inst2(LDR, X8, Ptr(X8, 1)),
            Some(Type::I32) => Assembly::inst2(LDR, X8, Ptr(X8, 4)),
            _ => Assembly::inst2(LDR, X8, Ptr(X8, 8)),
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
        .replace('@', "L_")
    }
}
