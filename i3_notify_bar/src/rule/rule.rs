use std::convert::TryFrom;

use i3_bar_components::protocol::Block;
use notify_server::notification::Notification;

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
    Ignore
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