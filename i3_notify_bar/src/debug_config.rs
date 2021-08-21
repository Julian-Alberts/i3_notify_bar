use notify_server::notification::Notification;

use crate::{args::DebugConfig, emoji::EmojiMode, notification_bar::{NotificationEvent, NotificationManager}, rule::Definition};

pub fn debug_config(config: Vec<Definition>, emoji_mode: EmojiMode, debug_config: DebugConfig) {
    use notify_server::Observer;
    use notify_server::Event;

    let mut nm = NotificationManager::new(config, emoji_mode);

    let DebugConfig {
        app_icon,
        app_name,
        body,
        expire_timeout,
        id,
        summary,
        urgency: _
    } = debug_config;

    nm.on_notify(
        &Event::Notify(
            Notification::new(
                app_name, id, app_icon, summary, body, Vec::new(), std::collections::HashMap::new(), expire_timeout
            )
        )
    );

    nm.get_events().into_iter().for_each(|e| {
        let e = if let NotificationEvent::Add(e) = e {
            e
        } else {
            unreachable!()
        };

        let e = e.read().unwrap();

        println!(
            "id: {}\nexpire_timeout: {}\nicon: {}\ntext: {}\nstyle: {:#?}\nemoji_mode: {:#?}", 
            e.id, e.expire_timeout, e.icon, e.text, e.style, e.emoji_mode
        );

    });
}
