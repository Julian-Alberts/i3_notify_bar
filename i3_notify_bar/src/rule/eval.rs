use std::ops::ControlFlow;

use crate::{
    notification_bar::{NotificationData, NotificationTemplateData},
    rule::NotificationRuleData,
};

use super::Rule;

pub struct RuleExcutor {
    rules: Vec<Rule>,
}

impl RuleExcutor {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }
}

impl<MatchedRules: Default + MatchedRule> EvalRules<MatchedRules> for RuleExcutor {
    fn eval(
        &self,
        n: &notify_server::notification::Notification,
        notification_template_data: &mut NotificationTemplateData,
        notification_data: &mut NotificationData,
    ) -> MatchedRules {
        let mut rules = MatchedRules::default();
        execute_rules_inner(
            &self.rules,
            n,
            notification_template_data,
            notification_data,
            &mut rules,
        );
        rules
    }
}

fn execute_rules_inner<MatchedRules: MatchedRule + Default>(
    rules: &[Rule],
    n: &notify_server::notification::Notification,
    notification_template_data: &mut NotificationTemplateData,
    notification_data: &mut NotificationData,
    matched_rules: &mut MatchedRules,
) -> ControlFlow<ExecuteActionBreakReason> {
    let mut index = 0;
    for rule in rules {
        index += 1;
        let rule_data = NotificationRuleData {
            app_icon: &n.app_icon,
            app_name: &n.app_name,
            body: &n.body,
            expire_timeout: notification_data.expire_timeout,
            group: &notification_data.group,
            summary: &n.summary,
            urgency: &n.urgency,
        };
        if !rule.matches(&rule_data) {
            continue;
        };
        matched_rules.add_rule(index);
        let action_result = rule.actions.iter().try_for_each(|action| {
            action.exec_action(notification_data, notification_template_data)
        });

        if let ControlFlow::Break(reason) = action_result {
            return ControlFlow::Break(reason);
        }

        notification_data.style.extend(rule.style.clone());

        let sub_rule_result = execute_rules_inner(
            &rule.sub_rule,
            n,
            notification_template_data,
            notification_data,
            matched_rules,
        );
        if matches!(sub_rule_result, ControlFlow::Break(_)) {
            return sub_rule_result;
        }
    }
    ControlFlow::Continue(())
}

pub enum ExecuteActionBreakReason {
    Stop,
    Ignore,
}

#[mockall::automock]
pub trait EvalRules<MatchedRules: Default + MatchedRule> {
    fn eval(
        &self,
        n: &notify_server::notification::Notification,
        notification_template_data: &mut NotificationTemplateData,
        notification_data: &mut NotificationData,
    ) -> MatchedRules;
}

pub trait MatchedRule {
    type Inner: MatchedRule;
    fn add_rule(&mut self, rule_id: usize) -> &mut Self::Inner;
}

impl MatchedRule for () {
    type Inner = Self;

    fn add_rule(&mut self, _: usize) -> &mut Self::Inner {
        self
    }
}

#[derive(Default)]
pub struct MatchedRules {
    rules: Vec<MatchedRulesInner>,
}

impl MatchedRule for MatchedRules {
    type Inner = MatchedRulesInner;
    fn add_rule(&mut self, rule_id: usize) -> &mut Self::Inner {
        self.rules.push(MatchedRulesInner {
            rule_id,
            sub_rules: Vec::default(),
        });
        let rule = self.rules.last_mut();
        unsafe {
            std::hint::assert_unchecked(rule.is_some());
        }
        rule.unwrap()
    }
}

impl std::fmt::Display for MatchedRules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stack = Vec::default();
        for r in &self.rules {
            fmt_matched_rules(r, &mut stack, f)?;
        }
        Ok(())
    }
}

fn fmt_matched_rules(
    matched_rules: &MatchedRulesInner,
    stack: &mut Vec<usize>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    stack.push(matched_rules.rule_id);
    let mut stack_iter = stack.into_iter();
    let id = stack_iter.next();
    unsafe {
        std::hint::assert_unchecked(id.is_some());
    }
    write!(f, "{}", id.unwrap())?;
    for id in stack_iter {
        write!(f, ".{id}")?;
    }
    writeln!(f)?;

    matched_rules
        .sub_rules
        .iter()
        .try_for_each(|r| fmt_matched_rules(r, stack, f))?;
    stack.pop();
    Ok(())
}

pub struct MatchedRulesInner {
    rule_id: usize,
    sub_rules: Vec<MatchedRulesInner>,
}

impl MatchedRule for MatchedRulesInner {
    type Inner = MatchedRulesInner;
    fn add_rule(&mut self, rule_id: usize) -> &mut Self::Inner {
        self.sub_rules.push(MatchedRulesInner {
            rule_id,
            sub_rules: Vec::default(),
        });
        let rule = self.sub_rules.last_mut();
        unsafe {
            std::hint::assert_unchecked(rule.is_some());
        }
        rule.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use notify_server::notification::Urgency;

    use crate::{
        notification_bar::{NotificationData, NotificationTemplateData},
        rule::{Condition, IgnoreAction, Rule, SetAction, StopAction},
    };

    fn notification(id: impl Into<notify_server::NotificationId>) -> NotificationData {
        NotificationData {
            actions: Vec::default(),
            emoji_mode: emoji::EmojiMode::Ignore,
            expire_timeout: 10,
            group: None,
            icon: ' ',
            id: id.into(),
            ignore: false,
            notification_update_id: 1,
            remove_in_secs: None,
            style: Default::default(),
            text: Default::default(),
            #[cfg(feature = "audio")]
            notification_sound: None,
            #[cfg(feature = "audio")]
            volume: 1.,
        }
    }

    fn server_notification() -> notify_server::notification::Notification {
        notify_server::notification::Notification {
            app_name: "".into(),
            id: 0.into(),
            app_icon: "".into(),
            summary: "".into(),
            body: "".into(),
            urgency: Urgency::Normal,
            actions: vec![],
            expire_timeout: -1,
        }
    }
    fn notification_template() -> NotificationTemplateData {
        NotificationTemplateData {
            app_name: "".into(),
            icon: "".into(),
            summary: "".into(),
            body: "".into(),
            expire_timeout: 0,
            time: 0,
        }
    }

    #[test]
    fn execute_rule_ignore() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(
            &[Rule {
                actions: vec![Box::new(IgnoreAction)],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
            &mut (),
        );
        assert!(nd.ignore);
    }

    #[test]
    fn execute_rule_empty() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(&[], &n, &mut ntd, &mut nd, &mut ());
        assert!(!nd.ignore);
        assert!(nd.actions.is_empty());
        assert_eq!(nd.expire_timeout, 10);
        assert_eq!(nd.id, 0.into());
    }

    #[test]
    fn execute_rule_set_group() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(
            &[Rule {
                actions: vec![Box::new(SetAction {
                    set_property: Box::new(crate::rule::SetProperty::new(
                        "TestGroup",
                        |v, _| v.to_string(),
                        |d, v| d.group = Some(v),
                    )),
                })],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
            &mut (),
        );
        assert_eq!(nd.group, Some("TestGroup".into()));
    }

    #[test]
    fn execute_rule_stop() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(
            &[
                Rule {
                    actions: vec![Box::new(StopAction)],
                    ..Default::default()
                },
                Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            "TestGroup",
                            |v, _| v.to_string(),
                            |d, v| d.group = Some(v),
                        )),
                    })],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
            &mut (),
        );
        assert!(nd.group.is_none());
    }

    #[test]
    fn execute_rule_multiple() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(
            &[
                Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            'W',
                            |v, _| *v,
                            |d, v| d.icon = v,
                        )),
                    })],
                    ..Default::default()
                },
                Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: (Box::new(crate::rule::SetProperty::new(
                            "TestGroup",
                            |v, _| v.to_string(),
                            |d, v| d.group = Some(v),
                        ))),
                    })],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
            &mut (),
        );
        assert_eq!(nd.group, Some("TestGroup".into()));
        assert_eq!(nd.icon, 'W');
    }

    #[test]
    fn execute_rule_multiple_not_all_matching() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(
            &[
                Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            'W',
                            |v, _| *v,
                            |d, v| d.icon = v,
                        )),
                    })],
                    ..Default::default()
                },
                Rule {
                    conditions: vec![Box::new(Condition::<_, _, super::super::Eq>::new(
                        "other-name".to_string(),
                        |d| d.app_name,
                    ))],
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            "TestGroup",
                            |v, _| v.to_string(),
                            |d, v| d.group = Some(v),
                        )),
                    })],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
            &mut (),
        );
        assert_eq!(nd.group, None);
        assert_eq!(nd.icon, 'W');
    }

    #[test]
    fn execute_rule_sub_rule() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(
            &[Rule {
                actions: vec![Box::new(SetAction {
                    set_property: Box::new(crate::rule::SetProperty::new(
                        'W',
                        |v, _| *v,
                        |d, v| d.icon = v,
                    )),
                })],
                sub_rule: vec![Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            "TestGroup",
                            |v, _| v.to_string(),
                            |d, v| d.group = Some(v),
                        )),
                    })],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
            &mut (),
        );
        assert_eq!(nd.group, Some("TestGroup".into()));
        assert_eq!(nd.icon, 'W');
    }

    #[test]
    fn execute_rule_stop_in_sub_rule() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(
            &[
                Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            'W',
                            |v, _| *v,
                            |d, v| d.icon = v,
                        )),
                    })],
                    sub_rule: vec![Rule {
                        actions: vec![Box::new(StopAction)],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            "TestGroup",
                            |v, _| v.to_string(),
                            |d, v| d.group = Some(v),
                        )),
                    })],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
            &mut (),
        );
        assert!(nd.group.is_none());
        assert_eq!(nd.icon, 'W');
    }

    #[test]
    fn execute_rule_ignore_in_sub_rule() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(
            &[
                Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            'W',
                            |v, _| *v,
                            |d, v| d.icon = v,
                        )),
                    })],
                    sub_rule: vec![Rule {
                        actions: vec![Box::new(IgnoreAction)],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Rule {
                    actions: vec![Box::new(SetAction {
                        set_property: Box::new(crate::rule::SetProperty::new(
                            "TestGroup",
                            |v, _| v.to_string(),
                            |d, v| d.group = Some(v),
                        )),
                    })],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
            &mut (),
        );
        assert!(nd.ignore);
        assert!(nd.group.is_none());
        assert_eq!(nd.icon, 'W');
    }
}
