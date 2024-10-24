mod eval;

pub use crate::config_parser::parse_config;
use notify_server::notification::Urgency;
use regex::Regex;

use crate::{
    notification_bar::{NotificationData, NotificationTemplateData},
    template,
};
use emoji::{self, EmojiMode};
pub use eval::{EvalRules, RuleExcutor};

#[derive(Default)]
pub struct Config {
    pub rules: Vec<Rule>,
}

pub struct NotificationRuleData<'a> {
    pub app_icon: &'a str,
    pub app_name: &'a str,
    pub summary: &'a str,
    pub body: &'a str,
    pub group: Option<&'a str>,
    pub urgency: &'a notify_server::notification::Urgency,
    pub expire_timeout: i32,
}

#[derive(Default, Debug, PartialEq)]
pub struct Rule {
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub style: Vec<Style>,
    pub sub_rule: Vec<Rule>,
}

impl Rule {
    pub fn matches(&self, notification: &NotificationRuleData) -> bool {
        !self.conditions.iter().any(|r| !r.is_match(notification))
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Ignore,
    Set(SetProperty),
    Stop,
}

#[derive(Debug, PartialEq)]
pub enum SetProperty {
    Icon(char),
    Text(u64),
    ExpireTimeout(i32),
    EmojiMode(EmojiMode),
    Group(String),
}

impl SetProperty {
    pub fn set(&self, nd: &mut NotificationData, n: &NotificationTemplateData) {
        match self {
            Self::Icon(i) => nd.icon = *i,
            Self::Text(i) => {
                nd.text = emoji::handle(template::render_template(i, n), &nd.emoji_mode)
            }
            Self::ExpireTimeout(i) => {
                nd.expire_timeout = *i;
                nd.remove_in_secs = Some(*i as f64);
            }
            Self::EmojiMode(em) => nd.emoji_mode = em.clone(),
            Self::Group(g) => nd.group = Some(g.clone()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    AppName(String),
    AppIcon(String),
    Summary(ConditionTypeString),
    Body(ConditionTypeString),
    Group(ConditionTypeString),
    Urgency(NumberCondition<Urgency>),
    ExpireTimeout(NumberCondition),
}

impl Condition {
    fn is_match(&self, other: &NotificationRuleData) -> bool {
        match self {
            Condition::AppIcon(v) => v == other.app_icon,
            Condition::AppName(v) => v == other.app_name,
            Condition::Summary(ConditionTypeString::Literal(v)) => v == other.summary,
            Condition::Summary(ConditionTypeString::Regex(v)) => v.is_match(other.summary),
            Condition::Body(ConditionTypeString::Literal(v)) => v == other.body,
            Condition::Body(ConditionTypeString::Regex(v)) => v.is_match(other.body),
            Condition::Group(ConditionTypeString::Literal(v)) => v == other.group.unwrap_or(""),
            Condition::Group(ConditionTypeString::Regex(v)) => {
                v.is_match(other.group.unwrap_or(""))
            }
            Condition::Urgency(NumberCondition::Eq(v)) => *v == *other.urgency,
            Condition::Urgency(NumberCondition::Lt(v)) => *v > *other.urgency,
            Condition::Urgency(NumberCondition::Le(v)) => *v >= *other.urgency,
            Condition::Urgency(NumberCondition::Gt(v)) => *v < *other.urgency,
            Condition::Urgency(NumberCondition::Ge(v)) => *v <= *other.urgency,
                
            Condition::ExpireTimeout(NumberCondition::Eq(v)) => *v == other.expire_timeout,
            Condition::ExpireTimeout(NumberCondition::Lt(v)) => *v > other.expire_timeout,
            Condition::ExpireTimeout(NumberCondition::Le(v)) => *v >= other.expire_timeout,
            Condition::ExpireTimeout(NumberCondition::Gt(v)) => *v < other.expire_timeout,
            Condition::ExpireTimeout(NumberCondition::Ge(v)) => *v <= other.expire_timeout,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum NumberCondition<T=i32> {
    Eq(T),
    Lt(T),
    Le(T),
    Gt(T),
    Ge(T),
}

#[derive(Debug)]
pub enum ConditionTypeString {
    Literal(String),
    Regex(Regex),
}

impl PartialEq for ConditionTypeString {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(s), Self::Literal(o)) => s == o,
            (Self::Regex(s), Self::Regex(o)) => s.as_str() == o.as_str(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Background(String),
    Text(String),
}

impl Style {
    pub fn apply(&self, base_component: &mut impl i3_bar_components::components::prelude::Color) {
        match self {
            Style::Background(c) => base_component.set_color_background(Some(c.to_owned())),
            Style::Text(c) => base_component.set_color_text(Some(c.to_owned())),
        }
    }
}

mod from_config_file {
    use std::{borrow::Cow, str::FromStr};

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
            error: Box<dyn std::error::Error>
        },
        #[error("Unknown style property {0}")]
        UnknownStyleProperty(String),
        #[error("Unknown property {0}")]
        UnsupportedOperation(Cow<'static, str>, crate::config_parser::CompareOperation),
    }

    fn vec_try_into<T, O, E>(i: Vec<T>) -> Result<Vec<O>, E>
    where
        O: TryFrom<T, Error = E>,
    {
        i.into_iter().map(TryFrom::try_from).collect()
    }

    impl TryFrom<crate::config_parser::ConfigDef> for Config {
        type Error = Error;
        fn try_from(value: crate::config_parser::ConfigDef) -> Result<Self, Self::Error> {
            Ok(Self {
                rules: vec_try_into(value.rules)?,
            })
        }
    }

    impl TryFrom<crate::config_parser::RuleDef> for Rule {
        type Error = Error;
        fn try_from(value: crate::config_parser::RuleDef) -> Result<Self, Self::Error> {
            Ok(Self {
                conditions: vec_try_into(value.conditions)?,
                actions: vec_try_into(value.actions)?,
                style: vec_try_into(value.style)?,
                sub_rule: vec_try_into(value.sub_rules)?,
            })
        }
    }
    
    impl TryFrom<crate::config_parser::ConditionDef> for Condition {
        type Error = Error;
        fn try_from(value: crate::config_parser::ConditionDef) -> Result<Self, Self::Error> {
            use crate::config_parser::CompareOperation::*;

            let cond = match (value.property.0.as_str(), value.op) {
                ("body", Eq) => Condition::Body(ConditionTypeString::Literal(value.value.0)),
                ("body", Match) => Condition::Body(ConditionTypeString::Regex(
                    Regex::new(value.value.0.as_str()).map_err(|e| Error::ParseError { 
                        property: "body".into(), 
                        value: value.value.0, 
                        error: Box::new(e)
                    })?,
                )),
                ("body", op) => return Err(Error::UnsupportedOperation(Cow::from("body"), op)),

                ("group", Eq) => Condition::Group(ConditionTypeString::Literal(value.value.0)),
                ("group", op) => return Err(Error::UnsupportedOperation(Cow::from("group"), op)),

                ("app_name", Eq) => Condition::AppName(value.value.0),
                ("app_name", op) => {
                    return Err(Error::UnsupportedOperation(Cow::from("app_name"), op))
                }

                ("app_icon", Eq) => Condition::AppIcon(value.value.0),
                ("app_icon", op) => {
                    return Err(Error::UnsupportedOperation(Cow::from("app_icon"), op))
                }

                ("summary", Eq) => Condition::Summary(ConditionTypeString::Literal(value.value.0)),
                ("summary", Match) => Condition::Summary(ConditionTypeString::Regex(
                    Regex::new(value.value.0.as_str()).map_err(|e| Error::ParseError { 
                        property: "summary".into(), 
                        value: value.value.0, 
                        error: Box::new(e)
                    }
                )?)),
                ("summary", op) => {
                    return Err(Error::UnsupportedOperation(Cow::from("summary"), op))
                }

                ("urgency", Eq) => Condition::Urgency(NumberCondition::Eq(Urgency::from_str(value.value.0.as_str()).map_err(|e| Error::ParseError { 
                    property: "urgency".into(), 
                    value: value.value.0, 
                    error: Box::new(e)
                })?)),
                ("urgency", Lt) => Condition::Urgency(NumberCondition::Lt(Urgency::from_str(value.value.0.as_str()).map_err(|e| Error::ParseError { 
                    property: "urgency".into(), 
                    value: value.value.0, 
                    error: Box::new(e)
                })?)),
                ("urgency", Le) => Condition::Urgency(NumberCondition::Le(Urgency::from_str(value.value.0.as_str()).map_err(|e| Error::ParseError { 
                    property: "urgency".into(), 
                    value: value.value.0, 
                    error: Box::new(e)
                })?)),
                ("urgency", Ge) => Condition::Urgency(NumberCondition::Ge(Urgency::from_str(value.value.0.as_str()).map_err(|e| Error::ParseError { 
                    property: "urgency".into(), 
                    value: value.value.0, 
                    error: Box::new(e)
                })?)),
                ("urgency", Gt) => Condition::Urgency(NumberCondition::Gt(Urgency::from_str(value.value.0.as_str()).map_err(|e| Error::ParseError { 
                    property: "urgency".into(), 
                    value: value.value.0, 
                    error: Box::new(e)
                })?)),
                ("urgency", op) => {
                    return Err(Error::UnsupportedOperation(Cow::from("urgency"), op))
                }

                ("expire_timeout", Eq) => {
                    Condition::ExpireTimeout(NumberCondition::Eq(value.value.0.parse().map_err(|e| Error::ParseError { 
                        property: "expire_timeout".into(), 
                        value: value.value.0, 
                        error: Box::new(e)
                    })?))
                }
                ("expire_timeout", Lt) => {
                    Condition::ExpireTimeout(NumberCondition::Lt(value.value.0.parse().map_err(|e| Error::ParseError { 
                        property: "expire_timeout".into(), 
                        value: value.value.0, 
                        error: Box::new(e)
                    })?))
                }
                ("expire_timeout", Le) => {
                    Condition::ExpireTimeout(NumberCondition::Le(value.value.0.parse().map_err(|e| Error::ParseError { 
                        property: "expire_timeout".into(), 
                        value: value.value.0, 
                        error: Box::new(e)
                    })?))
                }
                ("expire_timeout", Ge) => {
                    Condition::ExpireTimeout(NumberCondition::Ge(value.value.0.parse().map_err(|e| Error::ParseError { 
                        property: "expire_timeout".into(), 
                        value: value.value.0, 
                        error: Box::new(e)
                    })?))
                }
                ("expire_timeout", Gt) => {
                    Condition::ExpireTimeout(NumberCondition::Gt(value.value.0.parse().map_err(|e| Error::ParseError { 
                        property: "expire_timeout".into(), 
                        value: value.value.0, 
                        error: Box::new(e)
                    })?))
                }
                ("expire_timeout", op) => {
                    return Err(Error::UnsupportedOperation(Cow::from("expire_timeout"), op))
                }
                _ => unreachable!(),
            };
            Ok(cond)
        }
    }

    impl TryFrom<crate::config_parser::ActionDef> for Action {
        type Error = Error;
        fn try_from(value: crate::config_parser::ActionDef) -> Result<Self, Self::Error> {
            use crate::config_parser::ActionDef;
            let action = match value {
                ActionDef::Stop => Self::Stop,
                ActionDef::Ignore => Self::Ignore,
                ActionDef::Set(set) => Self::Set(set.try_into()?),
            };
            Ok(action)
        }
    }

    impl TryFrom<crate::config_parser::ActionSetDef> for SetProperty {
        type Error = Error;
        fn try_from(value: crate::config_parser::ActionSetDef) -> Result<Self, Self::Error> {
            let set = match (value.property.0.as_str(), value.value.0) {
                ("expire_timeout", v) => Self::ExpireTimeout(v.parse().map_err(|e| Error::ParseError { 
                    property: "expire_timeout".into(), 
                    value: v, 
                    error: Box::new(e)
                })?),
                ("group", v) => Self::Group(v),
                ("icon", v) => Self::Icon(v.chars().next().unwrap_or('\0')),
                ("text", v) => Self::Text(template::add_template(v.clone()).map_err(|e| Error::ParseError { 
                    property: "expire_timeout".into(), 
                    value: v, 
                    error: Box::new(e)
                })?),
                ("emoji", v) if v == "ignore" => Self::EmojiMode(EmojiMode::Ignore),
                ("emoji", v) if v == "remove" => Self::EmojiMode(EmojiMode::Remove),
                ("emoji", v) if v == "replace" => Self::EmojiMode(EmojiMode::Replace),
                ("emoji", v) => {
                    return Err(Error::UnsupportedValue {
                        property: "emoji".into(),
                        value: v,
                    })
                }
                (k, _) => return Err(Error::UnknownProperty(k.into())),
            };
            Ok(set)
        }
    }

    impl TryFrom<crate::config_parser::StyleDef> for Style {
        type Error = Error;
        fn try_from(value: crate::config_parser::StyleDef) -> Result<Self, Self::Error> {
            let style = match value.property.0.as_str() {
                "text" => Self::Text(value.value.0),
                "background" => Self::Background(value.value.0),
                k => return Err(Error::UnknownStyleProperty(k.into())),
            };
            Ok(style)
        }
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use notify_server::notification::Urgency;

    use super::*;

    fn new_notification() -> NotificationRuleData<'static> {
        NotificationRuleData {
            app_icon: "",
            app_name: "",
            body: "",
            expire_timeout: 10,
            group: None,
            summary: "",
            urgency: &Urgency::Normal,
        }
    }

    #[test]
    fn definition_matches_all() {
        let mut n = new_notification();
        n.app_name = "test-app";
        let def = Rule {
            conditions: vec![
                Condition::AppName("test-app".to_owned()),
                Condition::ExpireTimeout(NumberCondition::Eq(10)),
            ],
            actions: Default::default(),
            style: Vec::default(),
            sub_rule: Vec::default(),
        };
        assert!(def.matches(&n))
    }

    #[test]
    fn definition_does_not_match() {
        let mut n = new_notification();
        n.app_name = "test-app";
        n.expire_timeout = 9;
        let def = Rule {
            conditions: vec![
                Condition::AppName("test-app".to_owned()),
                Condition::ExpireTimeout(NumberCondition::Eq(10)),
            ],
            actions: Default::default(),
            style: Vec::default(),
            sub_rule: Vec::default(),
        };
        assert!(!def.matches(&n))
    }

    mod action {

        mod set_property {
            use emoji::EmojiMode;
            use notify_server::notification::{Notification, Urgency};

            use crate::{
                notification_bar::{NotificationData, NotificationTemplateData},
                rule::SetProperty,
            };

            fn new_nd() -> NotificationData {
                NotificationData {
                    id: 0.into(),
                    actions: Default::default(),
                    emoji_mode: EmojiMode::Ignore,
                    notification_update_id: 0,
                    expire_timeout: -1,
                    remove_in_secs: None,
                    group: None,
                    icon: ' ',
                    ignore: false,
                    style: Vec::default(),
                    text: "Test Text".to_owned(),
                }
            }

            fn new_ntd() -> NotificationTemplateData {
                NotificationTemplateData::from(&Notification {
                    actions: Vec::default(),
                    app_icon: "".to_owned(),
                    app_name: "Test app".to_owned(),
                    body: "Test body".to_owned(),
                    expire_timeout: 10,
                    id: 0.into(),
                    summary: "Test summary".to_owned(),
                    urgency: Urgency::Normal,
                })
            }

            #[test]
            fn icon() {
                let icon = '#';
                let mut nd = new_nd();
                let prop = SetProperty::Icon(icon);
                let n = new_ntd();
                assert_ne!(icon, nd.icon);
                prop.set(&mut nd, &n);
                assert_eq!(icon, nd.icon);
            }

            #[test]
            fn text() {
                let text = "New Text";
                let template_id = crate::template::add_template(text.to_owned()).unwrap();
                let mut nd = new_nd();
                let prop = SetProperty::Text(template_id);
                let n = new_ntd();
                assert_ne!(text, nd.text);
                prop.set(&mut nd, &n);
                assert_eq!(text, nd.text);
            }

            #[test]
            fn expire_timeout() {
                let timeout = 100;
                let mut nd = new_nd();
                let prop = SetProperty::ExpireTimeout(timeout);
                let n = new_ntd();
                assert_ne!(timeout, nd.expire_timeout);
                assert!(nd.remove_in_secs.is_none());
                prop.set(&mut nd, &n);
                assert_eq!(timeout, nd.expire_timeout);
                assert_eq!(Some(timeout as f64), nd.remove_in_secs)
            }

            #[test]
            fn emoji_mode() {
                let emoji = EmojiMode::Remove;
                let mut nd = new_nd();
                let prop = SetProperty::EmojiMode(emoji.clone());
                let n = new_ntd();
                assert_ne!(emoji, nd.emoji_mode);
                prop.set(&mut nd, &n);
                assert_eq!(emoji, nd.emoji_mode);
            }

            #[test]
            fn group() {
                let group = "TestGroup";
                let mut nd = new_nd();
                let prop = SetProperty::Group(group.to_owned());
                let n = new_ntd();
                assert_ne!(Some(group), nd.group.as_deref());
                prop.set(&mut nd, &n);
                assert_eq!(Some(group), nd.group.as_deref());
            }
        }
    }

    mod style {
        use super::super::Style;
        use i3_bar_components::components::{prelude::Color, BaseComponent};

        #[test]
        fn set_background_color() {
            let color = "#FF00FF";
            let style = Style::Background(color.to_owned());
            let mut base = BaseComponent::new();
            assert_ne!(base.color_background(), Some(color));
            style.apply(&mut base);
            assert_eq!(base.color_background(), Some(color));
        }

        #[test]
        fn set_text_color() {
            let color = "#FF00FF";
            let style = Style::Text(color.to_owned());
            let mut base = BaseComponent::new();
            assert_ne!(base.color_text(), Some(color));
            style.apply(&mut base);
            assert_eq!(base.color_text(), Some(color));
        }
    }

    mod condition_match {

        use super::*;

        #[test]
        fn app_icon() {
            let condition = Condition::AppIcon(String::from("#"));
            let mut n = new_notification();
            n.app_icon = "#";
            assert!(condition.is_match(&n));
            n.app_icon = "";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn app_name() {
            let condition = Condition::AppName(String::from("name"));
            let mut n = new_notification();
            n.app_name = "name";
            assert!(condition.is_match(&n));
            n.app_name = "other";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn summary_literal() {
            let condition =
                Condition::Summary(ConditionTypeString::Literal(String::from("summary")));
            let mut n = new_notification();
            n.summary = "summary";
            assert!(condition.is_match(&n));
            n.summary = "other";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn summary_regex() {
            let condition =
                Condition::Summary(ConditionTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.summary = "summary";
            assert!(condition.is_match(&n));
            n.summary = "o2ther";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn body_literal() {
            let condition = Condition::Body(ConditionTypeString::Literal(String::from("body")));
            let mut n = new_notification();
            n.body = "body";
            assert!(condition.is_match(&n));
            n.body = "other";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn body_regex() {
            let condition =
                Condition::Body(ConditionTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.body = "body";
            assert!(condition.is_match(&n));
            n.body = "bo2dy";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn urgency_low() {
            let condition = Condition::Urgency("low".to_owned());
            let mut n = new_notification();
            n.urgency = &notify_server::notification::Urgency::Low;
            assert!(condition.is_match(&n));
            n.urgency = &notify_server::notification::Urgency::Normal;
            assert!(!condition.is_match(&n));
            n.urgency = &notify_server::notification::Urgency::Critical;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn urgency_normal() {
            let condition = Condition::Urgency("normal".to_owned());
            let mut n = new_notification();
            n.urgency = &notify_server::notification::Urgency::Low;
            assert!(!condition.is_match(&n));
            n.urgency = &notify_server::notification::Urgency::Normal;
            assert!(condition.is_match(&n));
            n.urgency = &notify_server::notification::Urgency::Critical;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn urgency_critical() {
            let condition = Condition::Urgency("critical".to_owned());
            let mut n = new_notification();
            n.urgency = &notify_server::notification::Urgency::Low;
            assert!(!condition.is_match(&n));
            n.urgency = &notify_server::notification::Urgency::Normal;
            assert!(!condition.is_match(&n));
            n.urgency = &notify_server::notification::Urgency::Critical;
            assert!(condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_eq() {
            let condition = Condition::ExpireTimeout(NumberCondition::Eq(42));
            let mut n = new_notification();
            n.expire_timeout = 42;
            assert!(condition.is_match(&n));
            n.expire_timeout = 21;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_lt() {
            let condition = Condition::ExpireTimeout(NumberCondition::Lt(10));
            let mut n = new_notification();
            n.expire_timeout = 9;
            assert!(condition.is_match(&n));
            n.expire_timeout = 10;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_le() {
            let condition = Condition::ExpireTimeout(NumberCondition::Le(10));
            let mut n = new_notification();
            n.expire_timeout = 9;
            assert!(condition.is_match(&n));
            n.expire_timeout = 10;
            assert!(condition.is_match(&n));
            n.expire_timeout = 11;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_gt() {
            let condition = Condition::ExpireTimeout(NumberCondition::Gt(10));
            let mut n = new_notification();
            n.expire_timeout = 11;
            assert!(condition.is_match(&n));
            n.expire_timeout = 10;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_ge() {
            let condition = Condition::ExpireTimeout(NumberCondition::Ge(10));
            let mut n = new_notification();
            n.expire_timeout = 10;
            assert!(condition.is_match(&n));
            n.expire_timeout = 11;
            assert!(condition.is_match(&n));
            n.expire_timeout = 9;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn condition_type_string_eq() {
            let regex = ConditionTypeString::Regex(Regex::new("").unwrap());
            let lit = ConditionTypeString::Literal("".to_owned());
            assert_eq!(regex, regex);
            assert_eq!(lit, lit);
            assert_ne!(regex, lit);
            assert_ne!(lit, regex);
        }

        #[test]
        fn group() {
            let mut condition = Condition::Group(ConditionTypeString::Literal("".to_owned()));
            let mut n = new_notification();
            assert!(n.group.is_none());
            assert!(condition.is_match(&n));

            condition = Condition::Group(ConditionTypeString::Literal("test".to_owned()));
            n.group = Some("test");
            assert!(condition.is_match(&n));

            condition = Condition::Group(ConditionTypeString::Regex(Regex::new("test").unwrap()));
            n.group = Some("test");
            assert!(condition.is_match(&n));
        }
    }
}
