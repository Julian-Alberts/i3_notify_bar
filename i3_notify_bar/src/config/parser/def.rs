use std::fmt::{Debug, Display};

#[derive(Debug, Default, PartialEq)]
pub struct ConfigDef {
    pub rules: Vec<RuleDef>,
    pub groups: Vec<GroupDef>,
}

#[derive(Debug, Default, PartialEq)]
pub struct RuleDef {
    pub conditions: Vec<ConditionDef>,
    pub actions: Vec<ActionDef>,
    pub style: Vec<StyleDef>,
    pub sub_rules: Vec<RuleDef>,
}

#[derive(Debug, Default, PartialEq)]
pub struct GroupDef {
    pub name: Option<PropertyName>,
    pub style: Vec<StyleDef>,
}

#[derive(Debug, PartialEq)]
pub struct ConditionDef {
    pub property: PropertyName,
    pub op: CompareOperation,
    pub value: Value,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PropertyName(pub String);
#[derive(Debug, Default, PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(i32),
    Urgency(String),
    #[default]
    Null,
    Color(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s:?}"),
            Self::Urgency(u) => Display::fmt(&u, f),
            Self::Number(n) => Display::fmt(&n, f),
            Self::Color(c) => Display::fmt(&c, f),
            Self::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CompareOperation {
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    Match,
}

#[derive(Debug, PartialEq)]
pub enum ActionDef {
    Set(ActionSetDef),
    Stop,
    Ignore,
}

#[derive(Debug, PartialEq)]
pub struct ActionSetDef {
    pub property: PropertyName,
    pub value: Value,
}

#[derive(Debug, Default, PartialEq)]
pub struct StyleDef {
    pub property: PropertyName,
    pub value: Value,
}
