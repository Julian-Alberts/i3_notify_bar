use std::str::FromStr;
use zbus::zvariant::Value;

use crate::NotificationId;

#[derive(Debug, Clone, PartialEq, jbe::Builder)]
pub struct Notification {
    #[builder({default: String::default()})]
    pub app_name: String,
    #[builder({default: 0.into()})]
    pub id: NotificationId,
    #[builder({default: String::default()})]
    pub app_icon: String,
    #[builder({default: String::default()})]
    pub summary: String,
    #[builder({default: String::default()})]
    pub body: String,
    #[builder({default: Urgency::Normal})]
    pub urgency: Urgency,
    #[builder({default: Vec::default()})]
    pub actions: Vec<Action>,
    #[builder({default: 0})]
    pub expire_timeout: i32,
}

unsafe impl Sync for Notification {}

#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash, Default)]
pub enum Urgency {
    Low = 0,
    #[default]
    Normal = 1,
    Critical = 2,
}

impl From<u8> for Urgency {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Low,
            1 => Self::Normal,
            2 => Self::Critical,
            _ => Default::default(),
        }
    }
}

impl From<Value<'_>> for Urgency {
    fn from(value: Value) -> Self {
        match value {
            Value::U8(v) => v.into(),
            _ => Default::default(),
        }
    }
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_int = *self as usize;
        let other_int = *other as usize;
        self_int.partial_cmp(&other_int)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub key: String,
    pub text: String,
}
