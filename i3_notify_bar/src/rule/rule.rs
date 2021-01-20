use std::convert::TryFrom;

use i3_bar_components::protocol::Block;
use notify_server::notification::{Notification, Urgency};

#[derive(Default)]
pub struct Definition {
    pub rules: Vec<Rule>,
    pub actions: Vec<Action>,
    pub style: Vec<Style>
}

impl Definition {

    pub fn matches(&self, notification: &Notification) -> bool {
        !self.rules.iter().any(|r| !r.eq(notification))
    }

}

pub enum Action {
    Ignore,
    Set(SetProperty)
}

impl TryFrom<&str> for Action {
    
    type Error = ();
    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let action = line.split_whitespace().next();
        let ok = match action {
            Some("ignore") => Self::Ignore,
            Some("set") => Self::Set(SetProperty::try_from(line)?),
            _ => return Err(())
        };
        Ok(ok)
    }

}

pub enum SetProperty {
    AppName(String),
    AppIcon(String),
    Summary(String),
    Body(String),
    Urgency(String),
    ExpireTimeout(i32)
}

impl SetProperty {

    pub fn set(&self, n: &mut Notification) {
        match self {
            Self::AppName(i) => n.app_name = i.to_owned(),
            Self::AppIcon(i) => n.app_icon = i.to_owned(),
            Self::Summary(i) => n.summary = i.to_owned(),
            Self::Body(i) => n.body = i.to_owned(),
            Self::Urgency(i) => match &i[..] {
                "low" => n.urgency = Urgency::Low,
                "normal" => n.urgency = Urgency::Normal,
                "critical" => n.urgency = Urgency::Critical,
                _ => {}
            },
            Self::ExpireTimeout(i) => n.expire_timeout = *i,
        }
    }

}

impl TryFrom<&str> for SetProperty {
    
    type Error = ();
    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        let value = parts[2..].join(" ");
        let ok = match parts.get(1) {
            Some(&"app_name") => Self::AppName(value),
            Some(&"app_icon") => Self::AppIcon(value),
            Some(&"summary") => Self::Summary(value),
            Some(&"body") => Self::Body(value),
            Some(&"urgency") => Self::Urgency(value),
            Some(&"expire_timeout") => Self::ExpireTimeout(value.parse().or(Err(()))?),
            _ => return Err(())
        };
        Ok(ok)
    }

}

pub enum Rule {
    AppName(String),
    AppIcon(String),
    Summary(String),
    Body(String),
    Urgency(String),
    ExpireTimeout(i32)
}

impl PartialEq<Notification> for Rule {

    fn eq(&self, other: &Notification) -> bool {
        match self {
            Rule::AppIcon(v) => v == &other.app_icon,
            Rule::AppName(v) => v == &other.app_name,
            Rule::Summary(v) => v == &other.summary,
            Rule::Body(v) => v == &other.body,
            Rule::Urgency(v) => {
                match &other.urgency {
                    notify_server::notification::Urgency::Low => v == "low" ,
                    notify_server::notification::Urgency::Normal => v == "normal" ,
                    notify_server::notification::Urgency::Critical => v == "critical"
                }
            },
            Rule::ExpireTimeout(v) => *v == other.expire_timeout 
        }
    }

}

#[derive(Clone)]
pub enum Style {
    Background(String),
    Text(String)
}

impl Style {
    
    pub fn apply(&self, block: &mut Block) {
        match self {
            Style::Background(c) => {
                block.set_background(c.to_owned())
            },
            Style::Text(c) => {
                block.set_color(c.to_owned())
            },

        }
    }

}

impl TryFrom<&str> for Style {

    type Error = ();

    fn try_from(line: &str) -> Result<Style, Self::Error> {
        let split = line.split_whitespace().collect::<Vec<&str>>();
        let name = split.get(0);

        let ok = match name {
            Some(&"background") => Self::Background(split.get(1).ok_or(())?.to_owned().to_owned()),
            Some(&"text") => Self::Text(split.get(1).ok_or(())?.to_owned().to_owned()),
            _ => {
                return Err(())
            }
        };

        Ok(ok)
    }

}