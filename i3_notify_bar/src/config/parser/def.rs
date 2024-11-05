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

#[derive(Debug, Default, PartialEq)]
pub struct PropertyName(pub String);
#[derive(Debug, Default, PartialEq)]
pub struct Value(pub String);

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
