use std::borrow::Cow;

use emoji::EmojiMode;
use notify_server::notification::Urgency;

use crate::config::parser::def::*;

use super::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown property {0}")]
    UnknownProperty(String),
    #[error("Unsupported value \"{value}\" for property \"{property}\"")]
    UnsupportedValue {
        property: Cow<'static, str>,
        value: String,
    },
    #[error("Unable to parse value \"{value}\" for property \"{property}\": {error}")]
    ParseError {
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
    fn try_from(value: ConfigDef) -> Result<Self, Self::Error> {
        Ok(Self {
            rules: vec_try_into(value.rules)?,
        })
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
            value: Value(value),
        }: ConditionDef,
    ) -> Result<Self, Self::Error> {
        match property.as_str() {
            "body" => body_cond(value, op),
            "group" => group_cond(value, op),
            "app_name" => app_name_cond(value, op),
            "app_icon" => app_icon_cond(value, op),
            "summary" => summary_cond(value, op),
            "urgency" => {
                let value = match value.as_str() {
                    "low" => Urgency::Low,
                    "normal" => Urgency::Normal,
                    "critical" => Urgency::Critical,
                    _ => {
                        return Err(Error::UnsupportedValue {
                            property: "urgency".into(),
                            value,
                        })
                    }
                };
                urgency_condition(value, op)
            }
            "expire_timeout" => {
                let value = value.parse().map_err(|e| Error::ParseError {
                    property: "expire_timeout".into(),
                    value,
                    error: Box::new(e),
                })?;
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
            let value = Regex::new(value.as_str()).map_err(|e| Error::ParseError {
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
        let set: Box<dyn SetProp + Send + Sync> = match (value.property.0.as_str(), value.value.0) {
            ("expire_timeout", v) => {
                let value = v.parse().map_err(|e| Error::ParseError {
                    property: "expire_timeout".into(),
                    value: v,
                    error: Box::new(e),
                })?;
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
                v,
                |v, _| v.clone(),
                |d, v| d.group = Some(v),
            )),
            ("icon", v) => Box::new(SetProperty::new(
                v.chars().next().unwrap_or('\0'),
                |v, _| *v,
                |d, v| d.icon = v,
            )),
            ("text", v) => {
                let template_id =
                    crate::template::add_template(v.clone()).map_err(|e| Error::ParseError {
                        property: "expire_timeout".into(),
                        value: v,
                        error: Box::new(e),
                    })?;
                Box::new(SetProperty::new(
                    template_id,
                    |v, t| crate::template::render_template(v, t),
                    |d, v| d.text = v,
                ))
            }
            ("emoji", v) => {
                let mode = match v.as_str() {
                    "ignore" => EmojiMode::Ignore,
                    "remove" => EmojiMode::Remove,
                    "replace" => EmojiMode::Replace,
                    _ => {
                        return Err(Error::UnsupportedValue {
                            property: "emoji".into(),
                            value: v,
                        })
                    }
                };
                Box::new(SetProperty::new(
                    mode,
                    |v, _| v.clone(),
                    |d, v| d.emoji_mode = v,
                ))
            }
            ("audio_file", v) => Box::new(SetProperty::new(
                std::path::PathBuf::from(v),
                |v, _| v.to_path_buf(),
                |d, v| d.audio = Some(v),
            )),
            (k, _) => return Err(Error::UnknownProperty(k.into())),
        };
        Ok(set)
    }
}

impl TryFrom<StyleDef> for Style {
    type Error = Error;
    fn try_from(value: StyleDef) -> Result<Self, Self::Error> {
        let style = match value.property.0.as_str() {
            "text" => Self::Text(value.value.0),
            "background" => Self::Background(value.value.0),
            k => return Err(Error::UnknownStyleProperty(k.into())),
        };
        Ok(style)
    }
}
