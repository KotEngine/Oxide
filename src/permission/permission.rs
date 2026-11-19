#[derive(Debug, Clone)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub default_value: PermissionDefault,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDefault {
    True,
    False,
    Op,
    NotOp,
}

impl PermissionDefault {
    pub fn is_allowed(self, is_op: bool) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
            Self::Op => is_op,
            Self::NotOp => !is_op,
        }
    }
}

impl Permission {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            default_value: PermissionDefault::Op,
            children: Vec::new(),
        }
    }
}
