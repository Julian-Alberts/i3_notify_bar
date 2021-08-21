use notify_server::notification::Notification;

use crate::{
    args::DebugConfig,
    emoji::EmojiMode,
    notification_bar::{execute_rules, NotificationData, NotificationTemplateData},
    rule::Definition,
};

pub fn debug_config(config: &Vec<Definition>, emoji_mode: EmojiMode, debug_config: DebugConfig) {
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

    let notification_template_data = NotificationTemplateData::from(&notification);

    let matched_rules = execute_rules(
        config,
        &notification,
        notification_template_data,
        &mut notification_data,
    );

    println!("##### Matched Rules #####");
    matched_rules.iter().for_each(|r| println!("{}", r));

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
