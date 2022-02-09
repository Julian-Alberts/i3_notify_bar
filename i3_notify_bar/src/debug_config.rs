use std::fmt::Display;

use notify_server::notification::Notification;

use crate::{
    args::DebugConfig,
    EmojiMode,
    notification_bar::{execute_rules, NotificationData, NotificationTemplateData},
    rule::Definition,
};

pub fn debug_config(config: &[Definition], emoji_mode: EmojiMode, debug_config: DebugConfig) {
    let DebugConfig {
        app_icon,
        app_name,
        body,
        expire_timeout,
        id,
        summary,
        urgency: _,
    } = debug_config;

    let notification = Notification::new(
        app_name,
        id,
        app_icon,
        summary,
        body,
        Vec::new(),
        std::collections::HashMap::new(),
        expire_timeout,
    );

    let mut notification_data = NotificationData::new(&notification, emoji_mode);

    let mut notification_template_data = NotificationTemplateData::from(&notification);

    let matched_rules = execute_rules(
        config,
        &notification,
        &mut notification_template_data,
        &mut notification_data,
    );
    drop(notification_template_data);

    println!("##### Matched Rules #####");
    println!("{matched_rules}");

    println!(
        "##### Notification #####\nid: {}\nexpire_timeout: {}\nicon: {}\ntext: {}\nstyle: {:#?}\nemoji_mode: {:#?}",
        notification_data.id,
        notification_data.expire_timeout,
        notification_data.icon,
        notification_data.text,
        notification_data.style,
        notification_data.emoji_mode
    );
}

pub struct MatchedDefinitionTree {
    id: Option<usize>,
    branches: Vec<MatchedDefinitionTree>
}

impl MatchedDefinitionTree {

    pub fn new(id: usize) -> Self {
        Self {
            id: Some(id),
            branches: Vec::new()
        }
    }

    pub fn new_root() -> Self {
        Self {
            id: None,
            branches: Vec::new()
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
            writeln!(f, "")?;
            self.branches.iter().try_for_each(|b| writeln!(f, "{b}"))
        }
    }

}
