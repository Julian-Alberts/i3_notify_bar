use std::fmt::Display;

use notify_server::notification::{NotificationBuilder, Urgency};

use crate::{
    args::DebugConfig,
    notification_bar::{NotificationData, NotificationTemplateData},
    rule::{Config, EvalRules, RuleExcutor},
    EmojiMode,
};

pub fn debug_config(config: Config, emoji_mode: EmojiMode, debug_config: DebugConfig) {
    let DebugConfig {
        app_icon,
        app_name,
        body,
        expire_timeout,
        id,
        summary,
        urgency: _,
    } = debug_config;

    let notification = NotificationBuilder::default()
        .with_app_name(app_name)
        .with_id(id.into())
        .with_app_icon(app_icon)
        .with_summary(summary)
        .with_body(body)
        .with_actions(Vec::default())
        .with_urgency(Urgency::default())
        .with_expire_timeout(expire_timeout)
        .build();

    let mut notification_data = NotificationData::new(&notification, emoji_mode);

    let mut notification_template_data = NotificationTemplateData::from(&notification);

    let rule_executor = RuleExcutor::new(config.rules);

    let matched_rules: crate::rule::MatchedRules = rule_executor.eval(
        &notification,
        &mut notification_template_data,
        &mut notification_data,
    );

    drop(notification_template_data);

    println!("##### Matched Rules #####");
    println!("{matched_rules}");

    println!("##### Notification #####\n {notification_data:#?}",);
}

pub struct MatchedDefinitionTree {
    id: Option<usize>,
    branches: Vec<MatchedDefinitionTree>,
}

impl MatchedDefinitionTree {
    pub fn new(id: usize) -> Self {
        Self {
            id: Some(id),
            branches: Vec::new(),
        }
    }

    pub fn new_root() -> Self {
        Self {
            id: None,
            branches: Vec::new(),
        }
    }

    pub fn add_branch(&mut self, branch: MatchedDefinitionTree) {
        self.branches.push(branch)
    }
}

///
///     1
///     |`1
///
///
impl Display for MatchedDefinitionTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.id {
            writeln!(f, "{id}")?;
            self.branches.iter().try_for_each(|b| writeln!(f, "|`{b}"))
        } else {
            writeln!(f)?;
            self.branches.iter().try_for_each(|b| writeln!(f, "{b}"))
        }
    }
}
