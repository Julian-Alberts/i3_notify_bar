use std::convert::TryFrom;

use i3_bar_components::protocol::Block;
use notify_server::notification::Notification;
use tinytemplate::TinyTemplate;

use crate::notification_bar::NotificationData;

static mut TEMPLATE_MANAGER: Option<TinyTemplate<'static>> = None;
static mut TEMPLATES: Vec<String> = Vec::new();

fn get_template_manager() -> &'static TinyTemplate<'static> {
    unsafe {
        match &TEMPLATE_MANAGER {
            Some(tm) => tm,
            None => {
                TEMPLATE_MANAGER = Some(TinyTemplate::new());
                TEMPLATE_MANAGER.as_ref().unwrap()
            }
        }
    }
}

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
    Icon(String),
    Text(&'static str),
    ExpireTimeout(i32)
}

impl SetProperty {

    pub fn set(&self, nd: &mut NotificationData, n: &Notification) {
        match self {
            Self::Icon(i) => nd.icon = i.to_owned(),
            Self::Text(i) => nd.text = get_template_manager().render(i, n).unwrap(),
            Self::ExpireTimeout(i) => nd.expire_timeout = *i,
        }
    }

}

impl TryFrom<&str> for SetProperty {
    
    type Error = ();
    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        let value = parts[2..].join(" ");
        let ok = match parts.get(1) {
            Some(&"icon") => Self::Icon(value),
            Some(&"text") => Self::Text(property_template(value)?),
            Some(&"expire_timeout") => Self::ExpireTimeout(value.parse().or(Err(()))?),
            _ => return Err(())
        };
        Ok(ok)
    }

}

fn property_template(template: String) -> Result<&'static str, ()> {
    unsafe {
        TEMPLATES.push(template);
        let temp_ref = TEMPLATES.last().unwrap();
        match &mut TEMPLATE_MANAGER {
            Some(tm) => tm.add_template(temp_ref, temp_ref).or(Err(()))?,
            None => {
                let mut tm = TinyTemplate::new();
                tm.add_template(temp_ref, temp_ref).or(Err(()))?;
                TEMPLATE_MANAGER = Some(tm);
            }
        }
        Ok(temp_ref)
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