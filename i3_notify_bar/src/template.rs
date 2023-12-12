use std::sync::{OnceLock, RwLock};

use crate::notification_bar::NotificationTemplateData;

use chrono::{LocalResult, TimeZone};
use mini_template::{MiniTemplate, MiniTemplateBuilder};

pub const DEFAULT_TEMPLATE_ID: u64 = 0;

static mut TEMPLATE_MANAGER: OnceLock<RwLock<MiniTemplate>> = OnceLock::new();
static NEXT_TEMPLATE_ID: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(DEFAULT_TEMPLATE_ID);

pub fn render_template(tpl_id: &u64, context: &NotificationTemplateData) -> String {
    unsafe {
        let tplm = TEMPLATE_MANAGER
            .get_or_init(init_template_manager)
            .read()
            .unwrap_or_else(|e| e.into_inner());
        let output = match tplm.render(tpl_id.to_string().as_str(), context.clone().into()) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        };
        output.replace('\n', "")
    }
}

pub fn add_template(template: String) -> Result<u64, ()> {
    unsafe {
        let id = NEXT_TEMPLATE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let id_str = id.to_string();
        let old_template = TEMPLATE_MANAGER
            .get_or_init(init_template_manager)
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .add_template(id_str, template);

        match old_template {
            Ok(_) => Ok(id),
            Err(_) => Err(()), // TODO return better error
        }
    }
}

fn init_template_manager() -> RwLock<MiniTemplate> {
    let mut tplm = MiniTemplateBuilder::default()
        .with_default_modifiers()
        .with_modifier("date_time", &date_modifier)
        .with_modifier("max_len", &max_len)
        .build();
    if tplm
        .add_template(
            "0".to_owned(),
            "[{{app_name}}] {{summary}}: {{body}}".to_owned(),
        )
        .is_err()
    {
        unreachable!("Invalid default template")
    }
    RwLock::new(tplm)
}

#[mini_template::macros::create_modifier]
fn date_modifier(time: i64, format: Option<&str>) -> String {
    let LocalResult::Single(time) = chrono::Local.timestamp_opt(time, 0) else {
        return format!("Error while reading time UNIX time <{time}>");
    };
    if let Some(fmt) = format {
        time.format(fmt).to_string()
    } else {
        time.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
    }
}

#[mini_template::macros::create_modifier]
fn max_len(text: String, len: usize) -> String {
    if text.len() > len {
        text[..len].to_string()
    } else {
        text
    }
}
