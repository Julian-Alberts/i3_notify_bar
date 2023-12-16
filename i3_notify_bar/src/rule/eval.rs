use std::ops::ControlFlow;

use crate::{
    notification_bar::{NotificationData, NotificationTemplateData},
    rule::{Action, NotificationRuleData},
};

use super::Definition;

pub struct RuleExcutor {
    rules: Vec<Definition>,
}

impl RuleExcutor {
    pub fn new(rules: Vec<Definition>) -> Self {
        Self { rules }
    }
}

impl EvalRules for RuleExcutor {
    fn eval(
        &self,
        n: &notify_server::notification::Notification,
        notification_template_data: &mut NotificationTemplateData,
        notification_data: &mut NotificationData,
    ) {
        execute_rules_inner(
            &self.rules,
            n,
            notification_template_data,
            notification_data,
        );
    }
}

fn execute_rules_inner(
    definitions: &[Definition],
    n: &notify_server::notification::Notification,
    notification_template_data: &mut NotificationTemplateData,
    notification_data: &mut NotificationData,
) -> ControlFlow<ExecuteActionBreakReason> {
    for rule in definitions {
        use ExecuteActionBreakReason::*;
        let rule_data = NotificationRuleData {
            app_icon: &n.app_icon,
            app_name: &n.app_name,
            body: &n.body,
            expire_timeout: notification_data.expire_timeout,
            group: notification_data.group.as_deref(),
            summary: &n.summary,
            urgency: &n.urgency,
        };
        if !rule.matches(&rule_data) {
            continue;
        };
        let action_result = rule.actions.iter().try_for_each(|action| {
            excute_action(action, notification_data, notification_template_data)
        });

        match action_result {
            ControlFlow::Break(Stop) => return ControlFlow::Break(Stop),
            ControlFlow::Break(Ignore) => return ControlFlow::Break(Ignore),
            ControlFlow::Continue(_) => {}
        }

        notification_data.style.extend(rule.style.clone());

        let sub_rule_result = execute_rules_inner(
            &rule.sub_definition,
            n,
            notification_template_data,
            notification_data,
        );
        if matches!(sub_rule_result, ControlFlow::Break(_)) {
            return sub_rule_result;
        }
    }
    ControlFlow::Continue(())
}

enum ExecuteActionBreakReason {
    Stop,
    Ignore,
}

#[mockall::automock]
pub trait EvalRules {
    fn eval(
        &self,
        n: &notify_server::notification::Notification,
        notification_template_data: &mut NotificationTemplateData,
        notification_data: &mut NotificationData,
    );
}

fn excute_action(
    action: &Action,
    notification_data: &mut NotificationData,
    notification_template_data: &mut NotificationTemplateData,
) -> ControlFlow<ExecuteActionBreakReason> {
    use ExecuteActionBreakReason::*;
    match action {
        Action::Ignore => {
            notification_data.ignore = true;
            ControlFlow::Break(Ignore)
        }
        Action::Set(set_property) => {
            set_property.set(notification_data, notification_template_data);
            ControlFlow::Continue(())
        }
        Action::Stop => ControlFlow::Break(Stop),
    }
}

#[cfg(test)]
mod tests {
    use notify_server::notification::Urgency;

    use crate::{
        notification_bar::{NotificationData, NotificationTemplateData},
        rule::{Action, Conditions, Definition},
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
            &[Definition {
                actions: vec![Action::Ignore],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert!(nd.ignore);
    }

    #[test]
    fn execute_rule_empty() {
        let n = server_notification();
        let mut ntd = notification_template();
        let mut nd = notification(0);
        super::execute_rules_inner(&[], &n, &mut ntd, &mut nd);
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
            &[Definition {
                actions: vec![Action::Set(crate::rule::SetProperty::Group(
                    "TestGroup".into(),
                ))],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
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
                Definition {
                    actions: vec![Action::Stop],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
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
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
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
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                    ..Default::default()
                },
                Definition {
                    conditions: vec![Conditions::AppName("other name".to_string())],
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
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
            &[Definition {
                actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                sub_definition: vec![Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            &n,
            &mut ntd,
            &mut nd,
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
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                    sub_definition: vec![Definition {
                        actions: vec![Action::Stop],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
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
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Icon('W'))],
                    sub_definition: vec![Definition {
                        actions: vec![Action::Ignore],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Set(crate::rule::SetProperty::Group(
                        "TestGroup".into(),
                    ))],
                    ..Default::default()
                },
            ],
            &n,
            &mut ntd,
            &mut nd,
        );
        assert!(nd.ignore);
        assert!(nd.group.is_none());
        assert_eq!(nd.icon, 'W');
    }
}
