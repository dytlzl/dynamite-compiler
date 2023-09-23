use crate::aarch64::instruction::{InstOperand, InstOperator, Instruction};
use crate::Os;

pub enum Assembly {
    Inst(Instruction),
    Other(String),
    Group(Vec<Assembly>),
}

impl From<Instruction> for Assembly {
    fn from(val: Instruction) -> Self {
        Assembly::Inst(val)
    }
}

impl From<&'static str> for Assembly {
    fn from(val: &'static str) -> Self {
        Assembly::Other(String::from(val))
    }
}

impl From<String> for Assembly {
    fn from(val: String) -> Self {
        Assembly::Other(val)
    }
}

impl From<Vec<Assembly>> for Assembly {
    fn from(val: Vec<Assembly>) -> Self {
        Assembly::Group(val)
    }
}

impl Assembly {
    fn is_empty(&self) -> bool {
        match self {
            Assembly::Group(v) => v.is_empty(),
            _ => false,
        }
    }
    pub fn inst0(operator: InstOperator) -> Assembly {
        Assembly::Inst(Instruction {
            operator,
            ..Instruction::default()
        })
    }
    pub fn inst1<T1>(operator: InstOperator, operand1: T1) -> Assembly
    where
        T1: Into<InstOperand>,
    {
        Assembly::Inst(Instruction {
            operator,
            operand1: Some(operand1.into()),
            ..Instruction::default()
        })
    }
    pub fn inst2<T1, T2>(operator: InstOperator, operand1: T1, operand2: T2) -> Assembly
    where
        T1: Into<InstOperand>,
        T2: Into<InstOperand>,
    {
        Assembly::Inst(Instruction {
            operator,
            operand1: Some(operand1.into()),
            operand2: Some(operand2.into()),
            ..Instruction::default()
        })
    }
    pub fn inst3<T1, T2, T3>(
        operator: InstOperator,
        operand1: T1,
        operand2: T2,
        operand3: T3,
    ) -> Assembly
    where
        T1: Into<InstOperand>,
        T2: Into<InstOperand>,
        T3: Into<InstOperand>,
    {
        Assembly::Inst(Instruction {
            operator,
            operand1: Some(operand1.into()),
            operand2: Some(operand2.into()),
            operand3: Some(operand3.into()),
            ..Instruction::default()
        })
    }
    pub fn inst4<T1, T2, T3, T4>(
        operator: InstOperator,
        operand1: T1,
        operand2: T2,
        operand3: T3,
        operand4: T4,
    ) -> Assembly
    where
        T1: Into<InstOperand>,
        T2: Into<InstOperand>,
        T3: Into<InstOperand>,
        T4: Into<InstOperand>,
    {
        Assembly::Inst(Instruction {
            operator,
            operand1: Some(operand1.into()),
            operand2: Some(operand2.into()),
            operand3: Some(operand3.into()),
            operand4: Some(operand4.into()),
        })
    }
}

impl crate::generator::Assembly for Assembly {
    fn to_string(&self, target_os: Os) -> String {
        match self {
            Assembly::Inst(i) => i.to_string(target_os),
            Assembly::Other(o) => o.clone(),
            Assembly::Group(b) => b
                .iter()
                .filter(|a| !a.is_empty())
                .map(|a| a.to_string(target_os))
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }
}
