use std::borrow::Cow;

use emoji::EmojiMode;
use notify_server::notification::Urgency;

use crate::config::parser::def::*;

use super::*;

macro_rules! unsupported_type {
    ($prop:literal) => {
        |error| Error::UnsupportedType {
            property: $prop.into(),
            error,
        }
    };
}

#[derive(Debug, thiserror::Error)]
pub struct UnsupportedTypeError {
    expected_type: &'static str,
    found_type: &'static str,
}

impl std::fmt::Display for UnsupportedTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Expected value of type {}, found type {}",
            self.expected_type, self.found_type
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown property {0}")]
    UnknownProperty(String),
    #[error("Unsupported value \"{value}\" for property \"{property}\"")]
    UnsupportedValue {
        property: Cow<'static, str>,
        value: String,
    },
    #[error("{error} for property \"{property}\"")]
    UnsupportedType {
        property: Cow<'static, str>,
        error: UnsupportedTypeError,
    },
    #[error("Unable to parse value \"{value}\" for property \"{property}\": {error}")]
    Parse {
        property: Cow<'static, str>,
        value: String,
        error: Box<dyn std::error::Error>,
    },
    #[error("Unknown style property {0}")]
    UnknownStyleProperty(String),
    #[error("Unsupported operation {0:?}")]
    UnsupportedOperation(CompareOperation),
}

fn vec_try_into<T, O, E>(i: Vec<T>) -> Result<Vec<O>, E>
where
    O: TryFrom<T, Error = E>,
{
    i.into_iter().map(TryFrom::try_from).collect()
}

impl TryFrom<ConfigDef> for Config {
    type Error = Error;
    fn try_from(mut value: ConfigDef) -> Result<Self, Self::Error> {
        let default_group = value
            .groups
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, v)| if v.name.is_none() { Some(i) } else { None })
            .map(|i| {
                let g = value.groups.remove(i);
                g.try_into()
            })
            .transpose()?
            .map(|dg: (Option<String>, GroupConfig)| dg.1);
        let groups = value
            .groups
            .into_iter()
            .filter_map(|g| match g.try_into() {
                Ok((Some(key), value)) => Some(Ok((key, value))),
                Ok((None, _)) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(Self {
            rules: vec_try_into(value.rules)?,
            default_group,
            groups,
        })
    }
}

impl TryFrom<GroupDef> for (Option<String>, GroupConfig) {
    type Error = Error;
    fn try_from(value: GroupDef) -> Result<Self, Self::Error> {
        Ok((
            value.name.map(|n| n.0),
            GroupConfig {
                style: vec_try_into(value.style)?,
            },
        ))
    }
}

impl TryFrom<RuleDef> for Rule {
    type Error = Error;
    fn try_from(value: RuleDef) -> Result<Self, Self::Error> {
        Ok(Self {
            conditions: vec_try_into(value.conditions)?,
            actions: vec_try_into(value.actions)?,
            style: vec_try_into(value.style)?,
            sub_rule: vec_try_into(value.sub_rules)?,
        })
    }
}

impl TryFrom<ConditionDef> for Box<dyn CheckCondition + Send + Sync> {
    type Error = Error;
    fn try_from(
        ConditionDef {
            property: PropertyName(property),
            op,
            value,
        }: ConditionDef,
    ) -> Result<Self, Self::Error> {
        match property.as_str() {
            "body" => body_cond(value.try_into().map_err(unsupported_type!("body"))?, op),
            "group" => group_cond(value.try_into().map_err(unsupported_type!("group"))?, op),
            "app_name" => app_name_cond(value.try_into().map_err(unsupported_type!("group"))?, op),
            "app_icon" => app_icon_cond(value.try_into().map_err(unsupported_type!("group"))?, op),
            "summary" => summary_cond(value.try_into().map_err(unsupported_type!("group"))?, op),
            "urgency" => {
                let value = match String::try_from(value)
                    .map_err(unsupported_type!("urgency"))?
                    .as_str()
                {
                    "low" => Urgency::Low,
                    "normal" => Urgency::Normal,
                    "critical" => Urgency::Critical,
                    v => {
                        return Err(Error::UnsupportedValue {
                            property: "urgency".into(),
                            value: v.to_string(),
                        })
                    }
                };
                urgency_condition(value, op)
            }
            "expire_timeout" => {
                let value = value
                    .try_into()
                    .map_err(unsupported_type!("expire_timeout"))?;
                expire_timeout_cond(value, op)
            }
            _ => unreachable!(),
        }
    }
}

fn body_cond(
    value: String,
    op: CompareOperation,
) -> Result<Box<dyn CheckCondition + Send + Sync>, Error> {
    str_cond(value, op, |d| d.body)
}
fn group_cond(
    value: String,
    op: CompareOperation,
) -> Result<Box<dyn CheckCondition + Send + Sync>, Error> {
    str_cond(value, op, |d| {
        d.group.as_ref().map(String::as_str).unwrap_or_default()
    })
}
fn app_name_cond(
    value: String,
    op: CompareOperation,
) -> Result<Box<dyn CheckCondition + Send + Sync>, Error> {
    str_cond(value, op, |d| d.app_name)
}
fn app_icon_cond(
    value: String,
    op: CompareOperation,
) -> Result<Box<dyn CheckCondition + Send + Sync>, Error> {
    str_cond(value, op, |d| d.app_icon)
}
fn summary_cond(
    value: String,
    op: CompareOperation,
) -> Result<Box<dyn CheckCondition + Send + Sync>, Error> {
    str_cond(value, op, |d| d.summary)
}

fn str_cond(
    value: String,
    op: CompareOperation,
    get: for<'a> fn(&'a NotificationRuleData<'a>) -> &'a str,
) -> Result<Box<dyn CheckCondition + Send + Sync>, Error> {
    let cond: Box<dyn CheckCondition + Send + Sync> = match op {
        CompareOperation::Eq => Box::new(Condition::<_, _, Eq>::new(value, get)),
        CompareOperation::Match => {
            let value = Regex::new(value.as_str()).map_err(|e| Error::Parse {
                property: "body".into(),
                value,
                error: Box::new(e),
            })?;
            Box::new(Condition::<_, _, Match>::new(value, get))
        }
        op => return Err(Error::UnsupportedOperation(op)),
    };
    Ok(cond)
}
fn urgency_condition(
    value: Urgency,
    op: CompareOperation,
) -> Result<Box<dyn CheckCondition + Send + Sync>, Error> {
    let get: for<'a> fn(&'a NotificationRuleData<'a>) -> &'a Urgency = |d| d.urgency;
    let cond: Box<dyn CheckCondition + Send + Sync> = match op {
        CompareOperation::Lt => Box::new(Condition::<_, _, Lt>::new(value, get)),
        CompareOperation::Le => Box::new(Condition::<_, _, Le>::new(value, get)),
        CompareOperation::Eq => Box::new(Condition::<_, _, Eq>::new(value, get)),
        CompareOperation::Ge => Box::new(Condition::<_, _, Ge>::new(value, get)),
        CompareOperation::Gt => Box::new(Condition::<_, _, Gt>::new(value, get)),
        op => return Err(Error::UnsupportedOperation(op)),
    };
    Ok(cond)
}
fn expire_timeout_cond(
    value: i32,
    op: CompareOperation,
) -> Result<Box<dyn CheckCondition + Send + Sync>, Error> {
    let get: for<'a> fn(&'a NotificationRuleData<'a>) -> &'a i32 = |d| &d.expire_timeout;
    let cond: Box<dyn CheckCondition + Send + Sync> = match op {
        CompareOperation::Lt => Box::new(Condition::<_, _, Lt>::new(value, get)),
        CompareOperation::Le => Box::new(Condition::<_, _, Le>::new(value, get)),
        CompareOperation::Eq => Box::new(Condition::<_, _, Eq>::new(value, get)),
        CompareOperation::Ge => Box::new(Condition::<_, _, Ge>::new(value, get)),
        CompareOperation::Gt => Box::new(Condition::<_, _, Gt>::new(value, get)),
        op => return Err(Error::UnsupportedOperation(op)),
    };
    Ok(cond)
}

impl TryFrom<ActionDef> for Box<dyn ExecAction> {
    type Error = Error;
    fn try_from(value: ActionDef) -> Result<Self, Self::Error> {
        use ActionDef;
        let action: Box<dyn ExecAction> = match value {
            ActionDef::Stop => Box::new(StopAction),
            ActionDef::Ignore => Box::new(IgnoreAction),
            ActionDef::Set(set) => Box::new(SetAction {
                set_property: set.try_into()?,
            }),
        };
        Ok(action)
    }
}

impl TryFrom<ActionSetDef> for Box<dyn SetProp + Send + Sync> {
    type Error = Error;
    fn try_from(value: ActionSetDef) -> Result<Self, Self::Error> {
        let set: Box<dyn SetProp + Send + Sync> = match (value.property.0.as_str(), value.value) {
            ("expire_timeout", v) => {
                let value = v.try_into().map_err(unsupported_type!("expire_timeout"))?;
                Box::new(SetProperty::new(
                    value,
                    |v, _| *v,
                    |d, v| {
                        d.expire_timeout = v;
                        d.remove_in_secs = Some(v as f64)
                    },
                ))
            }
            ("group", v) => Box::new(SetProperty::new(
                String::try_from(v).map_err(unsupported_type!("group"))?,
                |v, _| v.clone(),
                |d, v| d.group = Some(v),
            )),
            ("icon", v) => Box::new(SetProperty::new(
                String::try_from(v)
                    .map_err(unsupported_type!("group"))?
                    .chars()
                    .next()
                    .unwrap_or('\0'),
                |v, _| *v,
                |d, v| d.icon = v,
            )),
            ("text", v) => {
                let template_id = crate::template::add_template(
                    v.clone().try_into().map_err(unsupported_type!("text"))?,
                )
                .map_err(|e| Error::Parse {
                    property: "text".into(),
                    value: v.to_string(),
                    error: Box::new(e),
                })?;
                Box::new(SetProperty::new(
                    template_id,
                    crate::template::render_template,
                    |d, v| d.text = v,
                ))
            }
            ("emoji", v) => {
                let mode = match v {
                    Value::String(v) if v == "ignore" => EmojiMode::Ignore,
                    Value::String(v) if v == "remove" => EmojiMode::Remove,
                    Value::String(v) if v == "replace" => EmojiMode::Replace,
                    _ => {
                        return Err(Error::UnsupportedValue {
                            property: "emoji".into(),
                            value: v.to_string(),
                        })
                    }
                };
                Box::new(SetProperty::new(
                    mode,
                    |v, _| v.clone(),
                    |d, v| d.emoji_mode = v,
                ))
            }
            #[cfg(feature = "audio")]
            ("notification_sound", v) => Box::new(SetProperty::new(
                std::path::PathBuf::from(v),
                |v, _| v.to_path_buf(),
                |d, v| d.notification_sound = Some(v),
            )),
            #[cfg(feature = "audio")]
            ("volume", v) => Box::new(SetProperty::new(
                v.parse::<f32>().map_err(|e| Error::Parse {
                    property: "volume".into(),
                    value: v,
                    error: Box::new(e),
                })?,
                |v, _| {
                    if *v > 100. {
                        1.
                    } else if *v < 0. {
                        0.
                    } else {
                        *v / 100.
                    }
                },
                |d, v| d.volume = v,
            )),
            (k, _) => return Err(Error::UnknownProperty(k.into())),
        };
        Ok(set)
    }
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "any",
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Urgency(_) => "urgency",
            Self::Color(_) => "color",
        }
    }
}

impl TryFrom<Value> for i32 {
    type Error = UnsupportedTypeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(n),
            v => Err(UnsupportedTypeError {
                expected_type: "number",
                found_type: v.type_name(),
            }),
        }
    }
}

impl TryFrom<Value> for Color {
    type Error = UnsupportedTypeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Color(c) => Ok(Color(c)),
            v => Err(UnsupportedTypeError {
                expected_type: "color",
                found_type: v.type_name(),
            }),
        }
    }
}

impl TryFrom<Value> for Option<i32> {
    type Error = UnsupportedTypeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(n) => Ok(Some(n)),
            Value::Null => Ok(None),
            v => Err(UnsupportedTypeError {
                expected_type: "number?",
                found_type: v.type_name(),
            }),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = UnsupportedTypeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(n) => Ok(n),
            v => Err(UnsupportedTypeError {
                expected_type: "string",
                found_type: v.type_name(),
            }),
        }
    }
}

impl TryFrom<Value> for Option<String> {
    type Error = UnsupportedTypeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(n) => Ok(Some(n)),
            Value::Null => Ok(None),
            v => Err(UnsupportedTypeError {
                expected_type: "string?",
                found_type: v.type_name(),
            }),
        }
    }
}

impl TryFrom<StyleDef> for Style {
    type Error = Error;
    fn try_from(value: StyleDef) -> Result<Self, Self::Error> {
        let v = value.value.try_into();
        let style = match value.property.0.as_str() {
            "text" => Self::Text(v.map_err(unsupported_type!("text"))?),
            "background" => Self::Background(v.map_err(unsupported_type!("background"))?),
            k => return Err(Error::UnknownStyleProperty(k.into())),
        };
        Ok(style)
    }
}
