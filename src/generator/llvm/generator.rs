use std::{collections::HashMap, vec};

use crate::{
    ast::{reserved_functions, Identifier, ProgramAst},
    ctype::Type,
    error,
    func::Func,
    global::{GlobalVariable, GlobalVariableData},
    node::{Node, NodeType},
};

#[derive(Debug)]
pub struct Options<'a> {
    register_number: &'a mut usize,
    register_map: &'a mut HashMap<usize, usize>,
    register_queue: &'a mut Vec<String>,
}

impl<'a> Options<'a> {
    fn new_register(&mut self) -> usize {
        *self.register_number += 1;
        self.register_queue
            .push(format!("%{}", self.register_number));
        *self.register_number
    }
    fn register_from_offset(&mut self, offset: usize) -> usize {
        let entry = self.register_map.entry(offset);
        let register = *entry.or_insert_with(|| *self.register_number + 1);
        if register > *self.register_number {
            *self.register_number = register;
        }
        register
    }
}

pub struct IrGenerator<'a> {
    error_logger: &'a dyn error::ErrorLogger,
}

impl<'a> IrGenerator<'a> {
    pub fn new(error_logger: &'a dyn error::ErrorLogger) -> Self {
        Self { error_logger }
    }
    pub fn generate(&self, ast: ProgramAst) -> String {
        [
            self.gen_string_literals(ast.string_literals),
            ast.global_variables
                .iter()
                .map(|(s, gv)| self.gen_global_variable(s, gv))
                .collect::<Vec<String>>(),
            ast.functions
                .iter()
                .map(|(name, f)| self.gen_func(name, f) + "\n")
                .collect::<Vec<String>>(),
            vec![
                "declare i32 @printf(ptr noundef, ...)".to_string(),
                "declare i32 @putchar(i8 noundef, ...)".to_string(),
                "declare i32 @exit(i8 noundef, ...)".to_string(),
            ],
        ]
        .concat()
        .join("\n")
    }

    fn gen_string_literals(&self, string_literals: &[String]) -> Vec<String> {
        string_literals
            .iter()
            .enumerate()
            .map(|(i, str)| {
                format!(
                    "@.str.{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1",
                    i,
                    str.replace("\\\\", "*").replace('\\', "").len() + 1,
                    str.replace("\\n", "\\0A")
                )
            })
            .collect::<Vec<String>>()
    }

    fn align(ty: &Type) -> usize {
        match ty {
            Type::Ptr(_) => 8,
            Type::Arr(child_ty, _) => Self::align(child_ty),
            _ => ty.size_of(),
        }
    }

    fn gen_global_variable(&self, name: &str, gv: &GlobalVariable) -> String {
        format!(
            "@{} = global {}, align {}",
            name,
            Self::gen_initializer_element(&gv.ty, gv.data.as_ref()),
            Self::align(&gv.ty),
        )
    }

    fn gen_type(ty: Type) -> String {
        match ty {
            Type::I8 => "i8".to_string(),
            Type::I32 => "i32".to_string(),
            Type::Ptr(child_ty) => format!("{}*", Self::gen_type(*child_ty)),
            Type::Arr(child_ty, size) => format!("[{} x {}]", size, Self::gen_type(*child_ty)),
            _ => todo!(),
        }
    }

    fn gen_type_with_opaque_ptr(ty: Type) -> String {
        match ty {
            Type::I8 => "i8".to_string(),
            Type::I32 => "i32".to_string(),
            Type::Ptr(_) => "ptr".to_string(),
            Type::Arr(child_ty, size) => format!("[{} x {}]", size, Self::gen_type(*child_ty)),
            _ => todo!(),
        }
    }

    fn gen_initializer_element(ty: &Type, data: Option<&GlobalVariableData>) -> String {
        match ty {
            Type::Arr(children_ty, size) => {
                if let Some(GlobalVariableData::Arr(v)) = data {
                    format!(
                        "{} [{}]",
                        Self::gen_type(ty.clone()),
                        v.iter()
                            .enumerate()
                            .filter(|(i, _)| i < size)
                            .map(|(_, d)| { Self::gen_initializer_element(children_ty, Some(d)) })
                            .collect::<Vec<String>>()
                            .join(", "),
                    )
                } else {
                    format!("{} null", Self::gen_type(ty.clone()))
                }
            }
            Type::Ptr(_) => format!("{} null", Self::gen_type_with_opaque_ptr(ty.clone())),
            Type::I8 | Type::I32 => format!(
                "{} {}",
                Self::gen_type(ty.clone()),
                if let Some(GlobalVariableData::Elem(s)) = data {
                    s
                } else {
                    "0"
                }
            ),
            Type::Func(_, _) => todo!(),
        }
    }

    fn gen_func(&self, name: &str, func: &Func) -> String {
        if func.body.is_none() {
            return String::new();
        }
        let Type::Func(_, return_ty) = &func.cty else {todo!()};
        let options = &mut Options {
            register_number: &mut func.args.len(),
            register_map: &mut HashMap::<usize, usize>::new(),
            register_queue: &mut vec!["?".to_string(); 10],
        };
        [
            vec![format!(
                "define {} @{}({}) {{",
                Self::gen_type(*return_ty.clone()),
                name,
                func.args
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| format!(
                        "{} noundef %{}",
                        Self::gen_type(arg.resolve_type().unwrap()),
                        i
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            )],
            func.args
                .iter()
                .enumerate()
                .map(|(i, node)| {
                    let register = options.new_register();
                    options.register_map.insert(node.offset.unwrap(), register);
                    format!(
                        "  %{} = alloca {}, align {}",
                        register,
                        Self::gen_type(node.resolve_type().unwrap()),
                        node.resolve_type().unwrap().size_of(),
                    ) + "\n"
                        + &format!(
                            "  store {} %{}, ptr %{}, align {}",
                            Self::gen_type(node.resolve_type().unwrap()),
                            i,
                            register,
                            node.resolve_type().unwrap().size_of(),
                        )
                })
                .collect::<Vec<String>>(),
            self.gen_node(func.body.as_ref().unwrap(), options),
            vec![
                format!("  ret {} 0", Self::gen_type(*return_ty.clone())), // default return value
                "}".to_string(),
            ],
        ]
        .concat()
        .join("\n")
    }

    fn gen_node(&self, node: &Node, options: &mut Options) -> Vec<String> {
        match node.nt {
            NodeType::DefVar => {
                return node
                    .children
                    .iter()
                    .map(|node| {
                        [
                            vec![format!(
                                "  %{} = alloca {}, align {}",
                                options.register_from_offset(
                                    node.lhs
                                        .as_ref()
                                        .unwrap_or_else(|| {
                                            self.error_logger.print_error_position(
                                                node.token.as_ref().unwrap().pos,
                                                &format!("lhs is None: {:?}", node),
                                            );
                                            unreachable!()
                                        })
                                        .offset
                                        .unwrap()
                                ),
                                Self::gen_type(node.resolve_type().unwrap()),
                                node.resolve_type().unwrap().size_of(),
                            )],
                            self.gen_node(node, options),
                        ]
                        .concat()
                    })
                    .collect::<Vec<Vec<String>>>()
                    .concat()
            }
            NodeType::CallFunc => {
                let Some(return_ty) = &node.cty else {panic!("{:?}", node.cty)};
                let args_types = reserved_functions()
                    .get(node.global_name.as_str())
                    .map(|v| {
                        let Identifier::Static(Type::Func(args, _)) = v else { unreachable!() };
                        args.iter()
                            .map(|arg_type| Self::gen_type_with_opaque_ptr(arg_type.clone()))
                            .collect::<Vec<String>>()
                    })
                    .unwrap_or(
                        node.args
                            .iter()
                            .map(|arg| {
                                Self::gen_type_with_opaque_ptr(arg.resolve_type().unwrap_or_else({
                                    || {
                                        self.error_logger.print_error_position(
                                            arg.token.as_ref().unwrap().pos,
                                            &format!("cannot resolve type: {:?}", arg),
                                        );
                                        unreachable!()
                                    }
                                }))
                            })
                            .collect::<Vec<String>>(),
                    );
                let args = node
                    .args
                    .iter()
                    .map(|node| self.gen_node(node, options))
                    .collect::<Vec<Vec<String>>>()
                    .concat();
                let args_passing = node
                    .args
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        format!(
                            "{} noundef {}",
                            args_types.get(node.args.len() - i - 1).unwrap_or(
                                &Self::gen_type_with_opaque_ptr(arg.resolve_type().unwrap())
                            ),
                            options.register_queue.pop().unwrap(),
                        )
                    })
                    .rev()
                    .collect::<Vec<String>>()
                    .join(", ");
                return [
                    args,
                    vec![format!(
                        "  %{} = call {} ({}) @{}({})",
                        options.new_register(),
                        Self::gen_type(return_ty.clone()),
                        [args_types, vec!["...".to_string()]].concat().join(", "),
                        node.global_name,
                        args_passing,
                    )],
                    // push the return value
                ]
                .concat();
            }
            NodeType::Block => {
                return node
                    .children
                    .iter()
                    .map(|node| self.gen_node(node, options))
                    .collect::<Vec<Vec<String>>>()
                    .concat()
            }
            NodeType::Return => {
                let lhs = self.gen_node(node.lhs.as_ref().unwrap(), options);
                *options.register_number += 1;
                return [
                    lhs,
                    vec![format!(
                        "  ret {} {}",
                        Self::gen_type(node.resolve_type().unwrap()),
                        options.register_queue.pop().unwrap(),
                    )],
                ]
                .concat();
            }
            NodeType::Num => {
                options.register_queue.push(node.value.unwrap().to_string());
                return vec![];
            }
            NodeType::Assign => {
                let rhs = self.gen_node(node.rhs.as_ref().unwrap(), options);
                let lhs = match node.lhs.as_ref().unwrap().nt {
                    NodeType::LocalVar => format!(
                        "%{}",
                        options.register_from_offset(
                            node.lhs
                                .as_ref()
                                .unwrap()
                                .offset
                                .unwrap_or_else(|| panic!("{:?}", node)),
                        )
                    ),
                    NodeType::GlobalVar => format!("@{}", node.lhs.as_ref().unwrap().global_name,),
                    _ => {
                        self.error_logger.print_error_position(
                            node.lhs.as_ref().unwrap().token.clone().unwrap().pos,
                            &format!("unexpected node: {:?}", node.lhs.as_ref().unwrap().nt),
                        );
                        unreachable!()
                    }
                };
                return [
                    rhs,
                    vec![format!(
                        "  store {} {}, ptr {}, align {}",
                        Self::gen_type(node.rhs.as_ref().unwrap().resolve_type().unwrap()),
                        options.register_queue.pop().unwrap(),
                        lhs,
                        node.lhs.as_ref().unwrap().resolve_type().unwrap().size_of(),
                    )],
                ]
                .concat();
            }
            NodeType::Addr => {
                options
                    .register_queue
                    .push(node.lhs.as_ref().unwrap().dest.to_string());
                return vec![];
            }
            NodeType::Deref => {
                return [
                    self.gen_node(node.lhs.as_ref().unwrap().rhs.as_ref().unwrap(), options),
                    self.gen_node(node.lhs.as_ref().unwrap().lhs.as_ref().unwrap(), options),
                    vec![format!(
                        "  %{} = getelementptr inbounds {}, ptr {}, i64 0, i64 {}",
                        options.new_register(),
                        Self::gen_type(
                            node.lhs
                                .as_ref()
                                .unwrap()
                                .lhs
                                .as_ref()
                                .unwrap()
                                .resolve_type()
                                .unwrap()
                        ),
                        options.register_queue.pop().unwrap(),
                        options.register_queue.pop().unwrap(),
                    )],
                ]
                .concat()
            }
            NodeType::LocalVar => {
                return vec![format!(
                    "  %{} = load {}, ptr %{}, align {}",
                    options.new_register(),
                    Self::gen_type(node.resolve_type().unwrap()),
                    options.register_from_offset(node.offset.unwrap()),
                    node.resolve_type().unwrap().size_of(),
                )]
            }
            NodeType::GlobalVar => {
                return vec![format!(
                    "  %{} = load {}, ptr @{}, align {}",
                    options.new_register(),
                    Self::gen_type(node.resolve_type().unwrap()),
                    node.global_name,
                    node.resolve_type().unwrap().size_of(),
                )]
            }
            NodeType::If => {
                let cond = self.gen_node(node.cond.as_ref().unwrap(), options);
                let cond_result_register = options.register_queue.pop().unwrap();
                let then_register = options.new_register();
                let then = self.gen_node(node.then.as_ref().unwrap(), options);
                let else_register = options.new_register();
                let els = node
                    .els
                    .as_ref()
                    .map(|n| self.gen_node(n, options))
                    .unwrap_or(vec![]);
                let end_register = options.new_register();
                return [
                    cond,
                    vec![format!(
                        "  br i1 {}, label %{}, label %{}",
                        cond_result_register, then_register, else_register
                    )],
                    vec![format!("\n{}:", then_register)],
                    then,
                    vec![format!("  br label %{}", end_register)],
                    vec![format!("\n{}:", else_register)],
                    els,
                    vec![format!("  br label %{}", end_register)],
                    vec![format!("\n{}:", end_register)],
                ]
                .concat();
            }
            _ => {}
        }

        let lhs = self.gen_node(
            node.rhs.as_ref().unwrap_or_else(|| panic!("{:?}", node)),
            options,
        );
        let rhs = self.gen_node(
            node.lhs.as_ref().unwrap_or_else(|| panic!("{:?}", node)),
            options,
        );

        let operation = match node.nt {
            NodeType::Add => "add nsw",
            NodeType::Sub => "sub nsw",
            NodeType::Mul => "mul nsw",
            NodeType::Div => "sdiv",
            NodeType::Mod => "srem",
            NodeType::Eq => "icmp eq",
            NodeType::Ne => "icmp ne",
            NodeType::Lt => "icmp slt",
            NodeType::Le => "icmp sle",
            NodeType::BitAnd => "and",
            NodeType::BitXor => "xor",
            NodeType::BitOr => "or",
            _ => "unknown",
        };
        let rhs_register = options
            .register_queue
            .pop()
            .expect("register queue is empty");
        let lhs_register = options
            .register_queue
            .pop()
            .expect("register queue is empty");
        [
            lhs,
            rhs,
            vec![match node.nt {
                NodeType::Add
                | NodeType::Sub
                | NodeType::Mul
                | NodeType::Div
                | NodeType::Mod
                | NodeType::Eq
                | NodeType::Ne
                | NodeType::Lt
                | NodeType::Le
                | NodeType::BitAnd
                | NodeType::BitXor
                | NodeType::BitOr => {
                    format!(
                        "  %{} = {} {} {}, {}",
                        options.new_register(),
                        operation,
                        Self::gen_type(node.resolve_type().unwrap_or_else(|| {
                            self.error_logger.print_error_position(
                                node.token.as_ref().unwrap().pos,
                                &format!("cannot resolve type: {:?}", node.lhs),
                            );
                            unreachable!()
                        })),
                        rhs_register,
                        lhs_register,
                    )
                }
                _ => {
                    self.error_logger.print_error_position(
                        node.token.as_ref().unwrap().pos,
                        &format!("unexpected node: {:?}", node.nt),
                    );
                    unreachable!();
                }
            }],
        ]
        .concat()
    }
}
