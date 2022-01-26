use std::{collections::HashMap, str::FromStr};
use zbus::zvariant::Value;

#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub urgency: Urgency,
    pub actions: Vec<Action>,
    pub expire_timeout: i32,
}

impl Notification {
    pub fn new(
        app_name: String,
        id: u32,
        app_icon: String,
        summary: String,
        body: String,
        mut actions_array: Vec<String>,
        hints: HashMap<String, Value>,
        expire_timeout: i32,
    ) -> Self {
        let mut urgency = Urgency::Normal;

        hints.into_iter().for_each(|(key, hint)| match &key[..] {
            "urgency" => urgency = get_urgency(hint),
            _ => {}
        });

        let mut actions = Vec::with_capacity(actions_array.len() / 2);

        while actions_array.len() > 0 {
            let key = actions_array.remove(0);
            if actions_array.len() == 0 {
                break;
            }
            let text = actions_array.remove(0);
            actions.push(Action { key, text });
        }

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

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum Urgency {
    Low = 0,
    Normal = 1,
    Critical = 2,
}

impl FromStr for Urgency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "low" => Urgency::Low,
            "normal" => Urgency::Normal,
            "critical" => Urgency::Critical,
            _ => return Err(format!("Can not convert {} to urgency", s)),
        })
    }
}

impl PartialOrd for Urgency {
    fn ge(&self, other: &Self) -> bool {
        (*self as usize) >= *other as usize
    }

    fn gt(&self, other: &Self) -> bool {
        (*self as usize) > *other as usize
    }

    fn le(&self, other: &Self) -> bool {
        (*self as usize) <= *other as usize
    }

    fn lt(&self, other: &Self) -> bool {
        (*self as usize) < *other as usize
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let result = if self > other {
            std::cmp::Ordering::Greater
        } else if self < other {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        };
        Some(result)
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

#[derive(Debug, Clone)]
pub struct Action {
    pub key: String,
    pub text: String,
}
