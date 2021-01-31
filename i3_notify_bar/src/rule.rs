use std::{convert::TryFrom, io::{BufRead, Error, ErrorKind}};

use log::{debug, info};
use notify_server::notification::Notification;
use tinytemplate::TinyTemplate;

use crate::notification_bar::{NotificationData, NotificationTemplateData};

static mut TEMPLATE_MANAGER: Option<TinyTemplate<'static>> = None;
static mut TEMPLATES: Vec<String> = Vec::new();

macro_rules! error {
    ($($arg:tt)*) => {
        Err(Error::new(ErrorKind::Other, format!($($arg)*)))
    };
}

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

fn get_template_manager_mut() -> &'static mut TinyTemplate<'static> {
    unsafe {
        match &mut TEMPLATE_MANAGER {
            Some(tm) => tm,
            None => {
                TEMPLATE_MANAGER = Some(TinyTemplate::new());
                TEMPLATE_MANAGER.as_mut().unwrap()
            }
        }
    }
}

pub fn parse_config(config: &mut dyn BufRead) -> std::io::Result<Vec<Definition>> {
    info!("Reading rules");
    let mut definitions = Vec::new();
    let mut def = None;
    let mut rules = None;
    let mut actions = None;
    let mut styles = None;

    let line_iter = config.lines().enumerate();

    for (line_num, line) in line_iter {
        let line = match line {
            Ok(l) => l,
            Err(e) => return error!("Reading line {} failed: {}", line_num, e.to_string())
        };
        let line = line.trim();
        match (line, &mut def, &mut rules, &mut actions, &mut styles) {
            ("def", None, None, None, None) => 
                def = Some(Definition::default()),
            ("enddef", Some(_), None, None, None) => {
                // def.unwarp can not fail, checked by condition
                definitions.push(def.unwrap());
                def = None
            },
            ("rule", Some(_), None, None, None) => 
                rules = Some(Vec::new()),
            ("endrule", Some(def), Some(_), None, None) => {
                // rules.unwarp can not fail, checked by condition
                def.rules = rules.unwrap();
                rules = None
            },
            ("action", Some(_), None, None, None) => 
                actions = Some(Vec::new()),
            ("endaction", Some(def), None, Some(_), None) => {
                // actions.unwarp can not fail, checked by condition
                def.actions = actions.unwrap();
                actions = None
            },
            ("style", Some(_), None, None, None) => 
                styles = Some(Vec::new()),
            ("endstyle", Some(def), None, None, Some(_)) => {
                // def.style can not fail, checked by condition
                def.style = styles.unwrap();
                styles = None
            }
            (rule_line, Some(_), Some(rules), None, None) => {
                let split = rule_line.splitn(2, '=');
                let split = split.collect::<Vec<&str>>();
                if split.len() != 2 {
                    return error!("Missing argument in line {}", line_num)
                }

                let r = Rule::try_from(rule_line);
                match r {
                    Ok(r) => rules.push(r),
                    _ => return error!("Could not parse line {} \"{}\"", line_num, rule_line)
                }
            },
            (action_line, Some(_), None, Some(actions), None) => {
                let r = Action::try_from(action_line);
                match r {
                    Ok(r) => actions.push(r),
                    Err(_) => return error!("Could not parse line {}", line_num)
                }
            },
            (style_line, Some(_), None, None, Some(styles)) => {
                let style = match Style::try_from(style_line) {
                    Ok(o) => o,
                    Err(_) => return error!("Could not parse line {} \"{}\"", line_num, style_line)
                };
                styles.push(style)
            },
            ("", _, _, _, _) => {},
            _ => return error!("Unknown error: Can not parse line {}", line_num)
        }

    }
    info!("Finished reading rules. Rules found {}", definitions.len());
    Ok(definitions)
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
    Set(SetProperty),
    Stop
}

impl TryFrom<&str> for Action {
    
    type Error = ();
    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let action = line.split_whitespace().next();
        let ok = match action {
            Some("ignore") => Self::Ignore,
            Some("set") => Self::Set(SetProperty::try_from(line)?),
            Some("stop") => Self::Stop,
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

    pub fn set(&self, nd: &mut NotificationData, n: &NotificationTemplateData) {
        match self {
            Self::Icon(i) => nd.icon = i.to_owned(),
            Self::Text(i) => nd.text = get_template_manager().render(i, n).unwrap().replace('\n', ""),
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
        get_template_manager_mut().add_template(temp_ref, temp_ref).or(Err(()))?;
            
        Ok(temp_ref)
    }
}

#[derive(Debug, PartialEq)]
pub enum Rule {
    AppName(String),
    AppIcon(String),
    Summary(String),
    Body(String),
    Urgency(String),
    ExpireTimeout(i32)
}

impl TryFrom<&str> for Rule {

    type Error = ();
    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let parts = line.split('=').collect::<Vec<&str>>();

        if parts.len() < 2 {
            debug!("Missing parameter");
            return Err(());
        }

        let name = match parts.get(0) {
            Some(s) => *s,
            _ => unreachable!("Who did you even get here?")
        };

        let value = parts[1..].join(" ").trim().to_owned();

        match name.trim() {
            "app_name" => Ok(Rule::AppName(value)),
            "app_icon" => Ok(Rule::AppIcon(value)),
            "summary" => Ok(Rule::Summary(value)),
            "body" => Ok(Rule::Body(value)),
            "urgency" => Ok(Rule::Urgency(value)),
            "expire_timeout" => Ok(Rule::ExpireTimeout(value.parse().or(Err(()))?)),
            n => {
                debug!("Unknown property {}", n);
                Err(())
            }
        }
    }

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

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Background(String),
    Text(String)
}

impl Style {
    
    pub fn apply(&self, base_component: &mut i3_bar_components::components::BaseComponent) {
        match self {
            Style::Background(c) => {
                base_component.set_background(c.to_owned())
            },
            Style::Text(c) => {
                base_component.set_color(c.to_owned())
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

#[cfg(test)]
mod tests {
    use core::convert::TryFrom;

    use super::{Rule, Style};

    #[test]
    fn style_background_from_str() {
        const COLOR: &str = "#FFFFFF";
        let expected = Style::Background(COLOR.to_owned());
        let actual = Style::try_from(&format!("background {}", COLOR)[..]);
        assert!(actual.is_ok(), r#"Error parsing "{} {}""#, "background", COLOR);
        let actual = actual.unwrap();
        assert_eq!(expected, actual);

        assert!(Style::try_from("background").is_err());
        assert!(Style::try_from("background ").is_err());
    }

    #[test]
    fn style_text_from_str() {
        const COLOR: &str = "#FFFFFF";
        let expected = Style::Text(COLOR.to_owned());
        let actual = Style::try_from(&format!("text {}", COLOR)[..]);
        assert!(actual.is_ok(), r#"Error parsing "{} {}""#, "text", COLOR);
        let actual = actual.unwrap();
        assert_eq!(expected, actual);

        assert!(Style::try_from("text").is_err());
        assert!(Style::try_from("text ").is_err());
    }

    #[test]
    fn style_try_from_unknown() {
        assert!(Style::try_from("unknown_option").is_err())
    }

    macro_rules! rule_try_from_macro {
        ($fn_name:ident $cnf_key:literal = $cnf_value:literal $type:tt) => {
            #[test]
            fn $fn_name() {
                let rule = Rule::try_from(&format!("{} = {}", $cnf_key, $cnf_value)[..]);
                assert!(rule.is_ok());
                let rule = rule.unwrap();
                assert_eq!(Rule::$type($cnf_value.to_owned()), rule)
            }
        };
    }

    rule_try_from_macro!(rule_try_from_app_name "app_name" = "test_app_name" AppName);
    rule_try_from_macro!(rule_try_from_app_icon "app_icon" = "icon" AppIcon);
    rule_try_from_macro!(rule_try_from_summary "summary" = "summ" Summary);
    rule_try_from_macro!(rule_try_from_body "body" = "test body" Body);
    rule_try_from_macro!(rule_try_from_urgency "urgency" = "low" Urgency);
    rule_try_from_macro!(rule_try_from_expire_timeout "expire_timeout" = 100 ExpireTimeout);


}