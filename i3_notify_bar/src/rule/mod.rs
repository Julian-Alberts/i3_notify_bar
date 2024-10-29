mod eval;
mod from_config_file;

use std::{fmt::Debug, ops::ControlFlow};

use eval::ExecuteActionBreakReason;
use regex::Regex;

use crate::notification_bar::{NotificationData, NotificationTemplateData};
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
    pub group: &'a Option<String>,
    pub urgency: &'a notify_server::notification::Urgency,
    pub expire_timeout: i32,
}

#[derive(Default)]
pub struct Rule {
    pub conditions: Vec<Box<dyn CheckCondition + Send + Sync>>,
    pub actions: Vec<Box<dyn ExecAction>>,
    pub style: Vec<Style>,
    pub sub_rule: Vec<Rule>,
}

impl Rule {
    pub fn matches(&self, notification: &NotificationRuleData) -> bool {
        !self.conditions.iter().any(|r| !r.is_true(notification))
    }
}

#[derive(Debug)]
pub struct IgnoreAction;

impl ExecAction for IgnoreAction {
    fn exec_action<'a>(
        &self,
        nd: &'a mut NotificationData,
        _: &NotificationTemplateData,
    ) -> ControlFlow<ExecuteActionBreakReason> {
        nd.ignore = true;
        ControlFlow::Break(ExecuteActionBreakReason::Ignore)
    }
}

#[derive(Debug)]
pub struct SetAction {
    pub(crate) set_property: Box<dyn SetProp + Send + Sync + 'static>,
}

impl ExecAction for SetAction {
    fn exec_action<'a>(
        &self,
        data: &'a mut NotificationData,
        template: &NotificationTemplateData,
    ) -> ControlFlow<ExecuteActionBreakReason> {
        self.set_property.set_prop(data, template);
        ControlFlow::Continue(())
    }
}

#[derive(Debug)]
pub struct StopAction;

impl ExecAction for StopAction {
    fn exec_action<'a>(
        &self,
        _: &'a mut NotificationData,
        _: &NotificationTemplateData,
    ) -> ControlFlow<ExecuteActionBreakReason> {
        ControlFlow::Break(ExecuteActionBreakReason::Stop)
    }
}

pub trait ExecAction: Send + Sync {
    fn exec_action<'a>(
        &self,
        data: &'a mut NotificationData,
        template: &NotificationTemplateData,
    ) -> ControlFlow<ExecuteActionBreakReason>;
}

#[derive(Debug)]
pub struct SetProperty<S, V>
where
    S: Debug,
    V: Debug,
{
    value: S,
    calc_value_fn: fn(&S, &NotificationTemplateData) -> V,
    set_prop_fn: for<'a> fn(&'a mut NotificationData, V),
}

impl<S, V> SetProperty<S, V>
where
    S: Debug,
    V: Debug,
{
    pub fn new(
        value: S,
        calc_value_fn: fn(&S, &NotificationTemplateData) -> V,
        set_prop_fn: for<'a> fn(&'a mut NotificationData, V),
    ) -> Self {
        Self {
            value,
            calc_value_fn,
            set_prop_fn,
        }
    }
}

impl<S, V> SetProp for SetProperty<S, V>
where
    S: Debug,
    V: Debug,
{
    fn set_prop<'a>(&self, data: &'a mut NotificationData, template: &NotificationTemplateData) {
        let value = (self.calc_value_fn)(&self.value, template);
        (self.set_prop_fn)(data, value)
    }
}

pub trait SetProp: Debug {
    fn set_prop<'a>(&self, data: &'a mut NotificationData, template: &NotificationTemplateData);
}

pub struct Eq;
pub struct Le;
pub struct Lt;
pub struct Gt;
pub struct Ge;
pub struct Match;

#[derive(Debug)]
pub struct Condition<A, B, Op>
where
    B: Debug,
    A: ?Sized,
{
    value: B,
    get_property_fn: for<'a> fn(&'a NotificationRuleData<'a>) -> &'a A,
    _p: std::marker::PhantomData<Op>,
}

impl<A, B, O> Condition<A, B, O>
where
    B: Debug,
    A: ?Sized,
{
    fn new(value: B, get_property_fn: for<'a> fn(&'a NotificationRuleData<'a>) -> &'a A) -> Self {
        Self {
            value,
            get_property_fn,
            _p: Default::default(),
        }
    }
}

impl<A, B> CheckCondition for Condition<A, B, Eq>
where
    B: Debug,
    A: ?Sized + PartialEq<B>,
{
    fn is_true(&self, data: &NotificationRuleData<'_>) -> bool {
        (self.get_property_fn)(data).eq(&self.value)
    }
}
impl<A, B> CheckCondition for Condition<A, B, Le>
where
    B: Debug,
    A: ?Sized + PartialOrd<B>,
{
    fn is_true(&self, data: &NotificationRuleData<'_>) -> bool {
        let p = (self.get_property_fn)(data);
        p.le(&self.value)
    }
}
impl<A, B> CheckCondition for Condition<A, B, Lt>
where
    B: Debug,
    A: ?Sized + PartialOrd<B>,
{
    fn is_true(&self, data: &NotificationRuleData<'_>) -> bool {
        let p = (self.get_property_fn)(data);
        p.lt(&self.value)
    }
}
impl<A, B> CheckCondition for Condition<A, B, Gt>
where
    B: Debug,
    A: ?Sized + PartialOrd<B>,
{
    fn is_true(&self, data: &NotificationRuleData<'_>) -> bool {
        let p = (self.get_property_fn)(data);
        p.gt(&self.value)
    }
}
impl<A, B> CheckCondition for Condition<A, B, Ge>
where
    B: Debug,
    A: ?Sized + PartialOrd<B>,
{
    fn is_true(&self, data: &NotificationRuleData<'_>) -> bool {
        let p = (self.get_property_fn)(data);
        p.ge(&self.value)
    }
}
impl CheckCondition for Condition<str, Regex, Match> {
    fn is_true(&self, data: &NotificationRuleData<'_>) -> bool {
        let p = (self.get_property_fn)(data);
        self.value.is_match(p)
    }
}

pub trait CheckCondition {
    fn is_true(&self, data: &NotificationRuleData) -> bool;
}

impl<A, B, O> PartialEq for Condition<A, B, O>
where
    B: Debug + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.get_property_fn == other.get_property_fn
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
            group: &None,
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
                Box::new(Condition::<str, String, Eq>::new(
                    "test-app".to_owned(),
                    |d| d.app_name,
                )),
                Box::new(Condition::<_, _, Eq>::new(10, |d| &d.expire_timeout)),
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
                Box::new(Condition::<str, String, super::Eq>::new(
                    "test-app".to_owned(),
                    |d| d.app_name,
                )),
                Box::new(Condition::<_, _, Eq>::new(10, |d| &d.expire_timeout)),
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
                config::parser::def::{ActionSetDef, PropertyName, Value},
                notification_bar::{NotificationData, NotificationTemplateData},
                rule::SetProp,
            };

            macro_rules! action_set_def {
                ($p: literal $value:expr) => {
                    ActionSetDef {
                        property: PropertyName($p.into()),
                        value: Value($value),
                    }
                };
            }

            type BoxedActionSet = Box<dyn SetProp + Send + Sync>;

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
                let prop = action_set_def!("icon" icon.to_string());
                let prop = BoxedActionSet::try_from(prop).unwrap();
                let n = new_ntd();
                assert_ne!(icon, nd.icon);
                prop.set_prop(&mut nd, &n);
                assert_eq!(icon, nd.icon);
            }

            #[test]
            fn text() {
                let text = "New Text";
                let mut nd = new_nd();
                let prop = action_set_def!("text" text.to_string());
                let prop = BoxedActionSet::try_from(prop).unwrap();
                let n = new_ntd();
                assert_ne!(text, nd.text);
                prop.set_prop(&mut nd, &n);
                assert_eq!(text, nd.text);
            }

            #[test]
            fn expire_timeout() {
                let timeout = 100;
                let mut nd = new_nd();
                let prop = action_set_def!("expire_timeout" timeout.to_string());
                let prop = BoxedActionSet::try_from(prop).unwrap();
                let n = new_ntd();
                assert_ne!(timeout, nd.expire_timeout);
                assert!(nd.remove_in_secs.is_none());
                prop.set_prop(&mut nd, &n);
                assert_eq!(timeout, nd.expire_timeout);
                assert_eq!(Some(timeout as f64), nd.remove_in_secs)
            }

            #[test]
            fn emoji_mode() {
                let emoji = EmojiMode::Remove;
                let mut nd = new_nd();
                let prop = action_set_def!("emoji" "remove".to_string());
                let prop = BoxedActionSet::try_from(prop).unwrap();
                let n = new_ntd();
                assert_ne!(emoji, nd.emoji_mode);
                prop.set_prop(&mut nd, &n);
                assert_eq!(emoji, nd.emoji_mode);
            }

            #[test]
            fn group() {
                let group = "TestGroup";
                let mut nd = new_nd();
                let prop = action_set_def!("group" group.to_string());
                let prop = BoxedActionSet::try_from(prop).unwrap();
                let n = new_ntd();
                assert_ne!(Some(group), nd.group.as_deref());
                prop.set_prop(&mut nd, &n);
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

        use crate::config::parser::def::{CompareOperation, ConditionDef, PropertyName, Value};

        use super::*;

        type BoxedCond = Box<dyn CheckCondition + Send + Sync>;

        macro_rules! cond_def {
            ($p: literal $op:ident $value:expr) => {
                ConditionDef {
                    property: PropertyName($p.into()),
                    op: CompareOperation::$op,
                    value: Value($value),
                }
            };
        }

        #[test]
        fn app_icon() {
            let condition = cond_def!("app_icon" Eq "#".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.app_icon = "#";
            assert!(condition.is_true(&n));
            n.app_icon = "";
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn app_name() {
            let condition = cond_def!("app_name" Eq "name".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.app_name = "name";
            assert!(condition.is_true(&n));
            n.app_name = "other";
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn summary_literal() {
            let condition = cond_def!("summary" Eq "summary".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.summary = "summary";
            assert!(condition.is_true(&n));
            n.summary = "other";
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn summary_regex() {
            let condition = cond_def!("summary" Match "^[a-z]+$".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.summary = "summary";
            assert!(condition.is_true(&n));
            n.summary = "o2ther";
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn body_literal() {
            let condition = cond_def!("body" Eq "body".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.body = "body";
            assert!(condition.is_true(&n));
            n.body = "other";
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn body_regex() {
            let condition = cond_def!("body" Match "^[a-z]+$".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.body = "body";
            assert!(condition.is_true(&n));
            n.body = "bo2dy";
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn urgency_low() {
            let condition = cond_def!("urgency" Eq "low".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.urgency = &notify_server::notification::Urgency::Low;
            assert!(condition.is_true(&n));
            n.urgency = &notify_server::notification::Urgency::Normal;
            assert!(!condition.is_true(&n));
            n.urgency = &notify_server::notification::Urgency::Critical;
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn urgency_normal() {
            let condition = cond_def!("urgency" Eq "normal".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.urgency = &notify_server::notification::Urgency::Low;
            assert!(!condition.is_true(&n));
            n.urgency = &notify_server::notification::Urgency::Normal;
            assert!(condition.is_true(&n));
            n.urgency = &notify_server::notification::Urgency::Critical;
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn urgency_critical() {
            let condition = cond_def!("urgency" Eq "critical".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.urgency = &notify_server::notification::Urgency::Low;
            assert!(!condition.is_true(&n));
            n.urgency = &notify_server::notification::Urgency::Normal;
            assert!(!condition.is_true(&n));
            n.urgency = &notify_server::notification::Urgency::Critical;
            assert!(condition.is_true(&n));
        }

        #[test]
        fn expire_timeout_eq() {
            let condition = cond_def!("expire_timeout" Eq "42".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.expire_timeout = 42;
            assert!(condition.is_true(&n));
            n.expire_timeout = 21;
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn expire_timeout_lt() {
            let condition = cond_def!("expire_timeout" Lt "10".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.expire_timeout = 9;
            assert!(condition.is_true(&n));
            n.expire_timeout = 10;
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn expire_timeout_le() {
            let condition = cond_def!("expire_timeout" Le "10".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.expire_timeout = 9;
            assert!(condition.is_true(&n));
            n.expire_timeout = 10;
            assert!(condition.is_true(&n));
            n.expire_timeout = 11;
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn expire_timeout_gt() {
            let condition = cond_def!("expire_timeout" Gt "10".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.expire_timeout = 11;
            assert!(condition.is_true(&n));
            n.expire_timeout = 10;
            assert!(!condition.is_true(&n));
        }

        #[test]
        fn expire_timeout_ge() {
            let condition = cond_def!("expire_timeout" Ge "10".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            n.expire_timeout = 9;
            assert!(!condition.is_true(&n));
            n.expire_timeout = 10;
            assert!(condition.is_true(&n));
            n.expire_timeout = 11;
            assert!(condition.is_true(&n));
        }

        #[test]
        fn group() {
            let condition = cond_def!("group" Eq "".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let mut n = new_notification();
            assert!(n.group.is_none());
            assert!(condition.is_true(&n));

            let condition = cond_def!("group" Eq "test".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let group = Some("test".to_string());
            n.group = &group;
            assert!(condition.is_true(&n));

            let condition = cond_def!("group" Match "test".to_string());
            let condition = BoxedCond::try_from(condition).unwrap();
            let group = Some("test".to_string());
            n.group = &group;
            assert!(condition.is_true(&n));
        }
    }
}
