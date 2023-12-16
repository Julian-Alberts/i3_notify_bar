mod eval;

pub use crate::config_parser::parse_config;
use regex::Regex;

use crate::{
    notification_bar::{NotificationData, NotificationTemplateData},
    template,
};
use emoji::{self, EmojiMode};
pub use eval::{EvalRules, RuleExcutor};

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
            Conditions::Group(ConditionTypeString::Literal(v)) => v == &other.group.unwrap_or(""),
            Conditions::Group(ConditionTypeString::Regex(v)) => {
                v.is_match(&other.group.unwrap_or(""))
            }
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
        let def = Definition {
            conditions: vec![
                Conditions::AppName("test-app".to_owned()),
                Conditions::ExpireTimeout(NumberCondition::Eq(10)),
            ],
            actions: Default::default(),
            style: Vec::default(),
            sub_definition: Vec::default(),
        };
        assert!(def.matches(&n))
    }

    #[test]
    fn definition_does_not_match() {
        let mut n = new_notification();
        n.app_name = "test-app";
        n.expire_timeout = 9;
        let def = Definition {
            conditions: vec![
                Conditions::AppName("test-app".to_owned()),
                Conditions::ExpireTimeout(NumberCondition::Eq(10)),
            ],
            actions: Default::default(),
            style: Vec::default(),
            sub_definition: Vec::default(),
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
                    expire_timeout: 10,
                    group: None,
                    icon: ' ',
                    ignore: false,
                    remove_in_secs: Some(10.),
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
                prop.set(&mut nd, &n);
                assert_eq!(timeout, nd.expire_timeout);
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
            let condition = Conditions::AppIcon(String::from("#"));
            let mut n = new_notification();
            n.app_icon = "#";
            assert!(condition.is_match(&n));
            n.app_icon = "";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn app_name() {
            let condition = Conditions::AppName(String::from("name"));
            let mut n = new_notification();
            n.app_name = "name";
            assert!(condition.is_match(&n));
            n.app_name = "other";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn summary_literal() {
            let condition =
                Conditions::Summary(ConditionTypeString::Literal(String::from("summary")));
            let mut n = new_notification();
            n.summary = "summary";
            assert!(condition.is_match(&n));
            n.summary = "other";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn summary_regex() {
            let condition =
                Conditions::Summary(ConditionTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.summary = "summary";
            assert!(condition.is_match(&n));
            n.summary = "o2ther";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn body_literal() {
            let condition = Conditions::Body(ConditionTypeString::Literal(String::from("body")));
            let mut n = new_notification();
            n.body = "body";
            assert!(condition.is_match(&n));
            n.body = "other";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn body_regex() {
            let condition =
                Conditions::Body(ConditionTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.body = "body";
            assert!(condition.is_match(&n));
            n.body = "bo2dy";
            assert!(!condition.is_match(&n));
        }

        #[test]
        fn urgency_low() {
            let condition = Conditions::Urgency("low".to_owned());
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
            let condition = Conditions::Urgency("normal".to_owned());
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
            let condition = Conditions::Urgency("critical".to_owned());
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
            let mut condition = Conditions::Group(ConditionTypeString::Literal("".to_owned()));
            let mut n = new_notification();
            assert!(n.group.is_none());
            assert!(condition.is_match(&n));

            condition = Conditions::Group(ConditionTypeString::Literal("test".to_owned()));
            n.group = Some("test");
            assert!(condition.is_match(&n));

            condition = Conditions::Group(ConditionTypeString::Regex(Regex::new("test").unwrap()));
            n.group = Some("test");
            assert!(condition.is_match(&n));
        }
    }
}
