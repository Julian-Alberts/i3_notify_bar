use notify_server::notification::Notification;
use regex::Regex;
pub use crate::config_parser::parse_config;

use crate::{
    notification_bar::{NotificationData, NotificationTemplateData},
    template,
};

#[derive(Default, Debug, PartialEq)]
pub struct Definition {
    pub rules: Vec<Rule>,
    pub actions: Vec<Action>,
    pub style: Vec<Style>,
}

impl Definition {
    pub fn matches(&self, notification: &Notification) -> bool {
        !self.rules.iter().any(|r| !r.is_match(notification))
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
    Id(String),
    Text(u64),
    ExpireTimeout(i32),
}

impl SetProperty {
    pub fn set(&self, nd: &mut NotificationData, n: &NotificationTemplateData) {
        match self {
            Self::Icon(i) => nd.icon = *i,
            Self::Text(i) => nd.text = template::render_template(i, n),
            Self::ExpireTimeout(i) => nd.expire_timeout = *i,
            Self::Id(i) => nd.id = i.clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Rule {
    AppName(String),
    AppIcon(String),
    Summary(RuleTypeString),
    Body(RuleTypeString),
    Urgency(String),
    ExpireTimeout(i32),
}

impl Rule {
    fn is_match(&self, other: &Notification) -> bool {
        match self {
            Rule::AppIcon(v) => v == &other.app_icon,
            Rule::AppName(v) => v == &other.app_name,
            Rule::Summary(RuleTypeString::Literal(v)) => v == &other.summary,
            Rule::Summary(RuleTypeString::Regex(v)) => v.is_match(&other.summary),
            Rule::Body(RuleTypeString::Literal(v)) => v == &other.body,
            Rule::Body(RuleTypeString::Regex(v)) => v.is_match(&other.body),
            Rule::Urgency(v) => match &other.urgency {
                notify_server::notification::Urgency::Low => v == "low",
                notify_server::notification::Urgency::Normal => v == "normal",
                notify_server::notification::Urgency::Critical => v == "critical",
            },
            Rule::ExpireTimeout(v) => *v == other.expire_timeout,
        }
    }
}

#[derive(Debug)]
pub enum RuleTypeString {
    Literal(String),
    Regex(Regex),
}

impl PartialEq for RuleTypeString {
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
    pub fn apply(&self, base_component: &mut i3_bar_components::components::BaseComponent) {
        match self {
            Style::Background(c) => base_component.set_background(c.to_owned()),
            Style::Text(c) => base_component.set_color(c.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rule_match {

        use super::*;

        fn new_notification() -> Notification {
            Notification {
                app_name: "".to_owned(),
                actions: vec![],
                app_icon: "".to_owned(),
                body: "".to_owned(),
                expire_timeout: 0,
                id: 0,
                summary: "".to_owned(),
                urgency: notify_server::notification::Urgency::Normal,
            }
        }

        #[test]
        fn app_icon() {
            let rule = Rule::AppIcon(String::from("#"));
            let mut n = new_notification();
            n.app_icon = String::from("#");
            assert!(rule.is_match(&n));
            n.app_icon = String::new();
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn app_name() {
            let rule = Rule::AppName(String::from("name"));
            let mut n = new_notification();
            n.app_name = String::from("name");
            assert!(rule.is_match(&n));
            n.app_name = String::from("other");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn summary_literal() {
            let rule = Rule::Summary(RuleTypeString::Literal(String::from("summary")));
            let mut n = new_notification();
            n.summary = String::from("summary");
            assert!(rule.is_match(&n));
            n.summary = String::from("other");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn summary_regex() {
            let rule = Rule::Summary(RuleTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.summary = String::from("summary");
            assert!(rule.is_match(&n));
            n.summary = String::from("o2ther");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn body_literal() {
            let rule = Rule::Body(RuleTypeString::Literal(String::from("body")));
            let mut n = new_notification();
            n.body = String::from("body");
            assert!(rule.is_match(&n));
            n.body = String::from("other");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn body_regex() {
            let rule = Rule::Body(RuleTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.body = String::from("body");
            assert!(rule.is_match(&n));
            n.body = String::from("bo2dy");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn urgency() {
            let rule = Rule::Urgency("low".to_owned());
            let mut n = new_notification();
            n.urgency = notify_server::notification::Urgency::Low;
            assert!(rule.is_match(&n));
            n.urgency = notify_server::notification::Urgency::Critical;
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn expire_timeout() {
            let rule = Rule::ExpireTimeout(42);
            let mut n = new_notification();
            n.expire_timeout = 42;
            assert!(rule.is_match(&n));
            n.expire_timeout = 21;
            assert!(!rule.is_match(&n));
        }
    }
}
