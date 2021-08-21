use serde::Serialize;
use std::{collections::HashMap, str::FromStr};
use zvariant::Value;

#[derive(Debug, Clone, Serialize)]
pub struct Notification {
    pub app_name: String,
    pub id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub urgency: Urgency,
    pub actions: Vec<String>,
    pub expire_timeout: i32,
}

impl Notification {
    pub fn new(
        app_name: String,
        id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, Value>,
        expire_timeout: i32,
    ) -> Self {
        let mut urgency = Urgency::Normal;

        hints.into_iter().for_each(|(key, hint)| match &key[..] {
            "urgency" => urgency = get_urgency(hint),
            _ => {}
        });

        Self {
            app_name,
            id,
            app_icon,
            summary,
            body,
            actions,
            urgency,
            expire_timeout,
        }
    }
}

unsafe impl Sync for Notification {}

#[derive(Debug, Clone, Serialize)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

impl FromStr for Urgency {

    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "low" => Urgency::Low,
            "normal" => Urgency::Normal,
            "critical" => Urgency::Critical,
            _ => return Err(format!("Can not convert {} to urgency", s))
        })
    }

}

fn get_urgency(value: Value) -> Urgency {
    match value {
        Value::U8(0) => Urgency::Low,
        Value::U8(1) => Urgency::Normal,
        Value::U8(2) => Urgency::Critical,
        _ => Urgency::Normal,
    }
}
