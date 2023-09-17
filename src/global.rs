use crate::ctype::Type;

pub struct GlobalVariable {
    pub ty: Type,
    pub data: Option<GlobalVariableData>,
}

pub enum GlobalVariableData {
    Elem(String),
    Arr(Vec<GlobalVariableData>),
}
