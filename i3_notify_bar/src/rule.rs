pub use crate::config_parser::parse_config;
use regex::Regex;

use crate::{
    notification_bar::{NotificationData, NotificationTemplateData},
    template,
};
use emoji::{self, EmojiMode};

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
pub struct Definition {
    pub conditions: Vec<Conditions>,
    pub actions: Vec<Action>,
    pub style: Vec<Style>,
    pub sub_definition: Vec<Definition>,
}

impl Definition {
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
            Self::ExpireTimeout(i) => nd.expire_timeout = *i,
            Self::EmojiMode(em) => nd.emoji_mode = em.clone(),
            Self::Group(g) => nd.group = Some(g.clone()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Conditions {
    AppName(String),
    AppIcon(String),
    Summary(ConditionTypeString),
    Body(ConditionTypeString),
    Group(ConditionTypeString),
    Urgency(String),
    ExpireTimeout(NumberCondition),
}

impl Conditions {
    fn is_match(&self, other: &NotificationRuleData) -> bool {
        match self {
            Conditions::AppIcon(v) => v == &other.app_icon,
            Conditions::AppName(v) => v == &other.app_name,
            Conditions::Summary(ConditionTypeString::Literal(v)) => v == &other.summary,
            Conditions::Summary(ConditionTypeString::Regex(v)) => v.is_match(&other.summary),
            Conditions::Body(ConditionTypeString::Literal(v)) => v == &other.body,
            Conditions::Body(ConditionTypeString::Regex(v)) => v.is_match(&other.body),
            Conditions::Group(ConditionTypeString::Literal(v)) => todo!(),
            Conditions::Group(ConditionTypeString::Regex(v)) => todo!(),
            Conditions::Urgency(v) => match &other.urgency {
                notify_server::notification::Urgency::Low => v == "low",
                notify_server::notification::Urgency::Normal => v == "normal",
                notify_server::notification::Urgency::Critical => v == "critical",
            },
            Conditions::ExpireTimeout(NumberCondition::Eq(v)) => *v == other.expire_timeout,
            Conditions::ExpireTimeout(NumberCondition::Lt(v)) => *v > other.expire_timeout,
            Conditions::ExpireTimeout(NumberCondition::Le(v)) => *v >= other.expire_timeout,
            Conditions::ExpireTimeout(NumberCondition::Gt(v)) => *v < other.expire_timeout,
            Conditions::ExpireTimeout(NumberCondition::Ge(v)) => *v <= other.expire_timeout,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum NumberCondition {
    Eq(i32),
    Lt(i32),
    Le(i32),
    Gt(i32),
    Ge(i32),
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    mod condition_match {

        use super::*;

        fn new_notification() -> Notification {
            Notification {
                app_name: "".to_owned(),
                actions: vec![],
                app_icon: "".to_owned(),
                body: "".to_owned(),
                expire_timeout: 0,
                id: 0.into(),
                summary: "".to_owned(),
                urgency: notify_server::notification::Urgency::Normal,
            }
        }

        #[test]
        fn app_icon() {
            let condition = Conditions::AppIcon(String::from("#"));
            let mut n = new_notification();
            n.app_icon = String::from("#");
            assert!(condition.is_match(&n));
            n.app_icon = String::new();
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn app_name() {
            let condition = Conditions::AppName(String::from("name"));
            let mut n = new_notification();
            n.app_name = String::from("name");
            assert!(condition.is_match(&n));
            n.app_name = String::from("other");
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn summary_literal() {
            let condition =
                Conditions::Summary(ConditionTypeString::Literal(String::from("summary")));
            let mut n = new_notification();
            n.summary = String::from("summary");
            assert!(condition.is_match(&n));
            n.summary = String::from("other");
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn summary_regex() {
            let condition =
                Conditions::Summary(ConditionTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.summary = String::from("summary");
            assert!(condition.is_match(&n));
            n.summary = String::from("o2ther");
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn body_literal() {
            let condition = Conditions::Body(ConditionTypeString::Literal(String::from("body")));
            let mut n = new_notification();
            n.body = String::from("body");
            assert!(condition.is_match(&n));
            n.body = String::from("other");
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn body_regex() {
            let condition =
                Conditions::Body(ConditionTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.body = String::from("body");
            assert!(condition.is_match(&n));
            n.body = String::from("bo2dy");
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn urgency() {
            let condition = Conditions::Urgency("low".to_owned());
            let mut n = new_notification();
            n.urgency = notify_server::notification::Urgency::Low;
            assert!(condition.is_match(&n));
            n.urgency = notify_server::notification::Urgency::Critical;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_eq() {
            let condition = Conditions::ExpireTimeout(NumberCondition::Eq(42));
            let mut n = new_notification();
            n.expire_timeout = 42;
            assert!(condition.is_match(&n));
            n.expire_timeout = 21;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_lt() {
            let condition = Conditions::ExpireTimeout(NumberCondition::Lt(10));
            let mut n = new_notification();
            n.expire_timeout = 9;
            assert!(condition.is_match(&n));
            n.expire_timeout = 10;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_le() {
            let condition = Conditions::ExpireTimeout(NumberCondition::Le(10));
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
            let condition = Conditions::ExpireTimeout(NumberCondition::Gt(10));
            let mut n = new_notification();
            n.expire_timeout = 11;
            assert!(condition.is_match(&n));
            n.expire_timeout = 10;
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn expire_timeout_ge() {
            let condition = Conditions::ExpireTimeout(NumberCondition::Ge(10));
            let mut n = new_notification();
            n.expire_timeout = 10;
            assert!(condition.is_match(&n));
            n.expire_timeout = 11;
            assert!(condition.is_match(&n));
            n.expire_timeout = 9;
            assert!(!condition.is_match(&n));
        }
    }
}
