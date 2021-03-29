use std::{convert::TryFrom, io::{BufRead, Error, ErrorKind, Result as IOResult}};

use log::{debug, info};
use notify_server::notification::Notification;
use regex::Regex;

use crate::{notification_bar::{NotificationData, NotificationTemplateData}, template};

macro_rules! error {
    ($($arg:tt)*) => {
        Err(Error::new(ErrorKind::Other, format!($($arg)*)))
    };
}

pub fn parse_config(config: &mut dyn BufRead) -> IOResult<Vec<Definition>> {
    info!("Reading rules experimental");
    let mut definitions = Vec::new();
    let lines = config
        .lines()
        .map(Result::unwrap)
        .map(trim_string)
        .enumerate()
        .filter(ignore_lines)
        .collect::<Vec<(usize, String)>>();

    let definition_blocks = find_blocks(&lines[..], "def", "enddef")?;
    
    for block in definition_blocks {
        let action_blocks = find_blocks(block, "action", "endaction")?
            .iter()
            .fold(Vec::new(), fold_blocks);
        let rule_blocks = find_blocks(block, "rule", "endrule")?
            .iter()
            .fold(Vec::new(), fold_blocks);
        let style_blocks = find_blocks(block, "style", "endstyle")?
            .iter()
            .fold(Vec::new(), fold_blocks);

        let mut actions = Vec::new();
        for (line_num, line) in action_blocks.iter() {
            match Action::try_from(&line[..]) {
                Ok(a) => actions.push(a),
                Err(_) => return error!(r#"Faild to read action "{}" in line {}"#, line, line_num)
            }
        }

        let mut rules = Vec::new();
        for (line_num, line) in rule_blocks.iter() {
            match Rule::try_from(&line[..]) {
                Ok(r) => rules.push(r),
                Err(_) => return error!(r#"Faild to read rule "{}" in line {}"#, line, line_num)
            }
        }

        let mut style = Vec::new();
        for (line_num, line) in style_blocks.iter() {
            match Style::try_from(&line[..]) {
                Ok(s) => style.push(s),
                Err(_) => return error!(r#"Faild to read style "{}" in line {}"#, line, line_num)
            }
        }

        definitions.push(
            Definition {
                actions,
                rules,
                style
            }
        )
    }
    
    Ok(definitions)
}

fn trim_string(str: String) -> String {
    str.trim().to_owned()
}

fn ignore_lines((_, line): &(usize, String)) -> bool {
    !(line.is_empty() || line.starts_with('#'))
}

fn find_blocks<'a>(lines: &'a [(usize, String)], start_key: &str, end_key: &str) -> IOResult<Vec<&'a [(usize, String)]>> {
    let mut blocks = Vec::new();
    
    let mut searched_key = start_key;
    let mut other_key = end_key;

    let mut match_count = 0;
    let mut start_index = 0;

    for (index, (line_num, line)) in lines.iter().enumerate() {
        if line == searched_key {
            match_count += 1;
            std::mem::swap(&mut searched_key, &mut other_key);
            if match_count & 1 == 1 {
                start_index = index;
            } else {
                blocks.push(&lines[start_index+1..index])
            }
        } else if line == other_key {
            return error!("Unexpected {} in line {}", other_key, line_num)
        }
    }

    Ok(blocks)
}

fn fold_blocks<'a>(mut target: Vec<&'a (usize, String)>, data: &&'a [(usize, String)]) -> Vec<&'a(usize, String)>{
    target.extend(data.iter());
    target
}

#[derive(Default, Debug)]
pub struct Definition {
    pub rules: Vec<Rule>,
    pub actions: Vec<Action>,
    pub style: Vec<Style>
}

impl Definition {

    pub fn matches(&self, notification: &Notification) -> bool {
        !self.rules.iter().any(|r| !r.is_match(notification))
    }

}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum SetProperty {
    Icon(char),
    Id(String),
    Text(u64),
    ExpireTimeout(i32)
}

impl SetProperty {

    pub fn set(&self, nd: &mut NotificationData, n: &NotificationTemplateData) {
        match self {
            Self::Icon(i) => nd.icon = *i,
            Self::Text(i) => nd.text = template::render_template(i, n),
            Self::ExpireTimeout(i) => nd.expire_timeout = *i,
            Self::Id(i) => nd.id = i.clone()
        }
    }

}

impl TryFrom<&str> for SetProperty {
    
    type Error = ();
    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        let value = parts[2..].join(" ");
        let ok = match parts.get(1) {
            Some(&"icon") => Self::Icon(value.chars().next().ok_or(())?),
            Some(&"id") => Self::Id(value),
            Some(&"text") => Self::Text(template::add_template(value)?),
            Some(&"expire_timeout") => Self::ExpireTimeout(value.parse().or(Err(()))?),
            _ => return Err(())
        };
        Ok(ok)
    }

}

#[derive(Debug, PartialEq)]
pub enum Rule {
    AppName(String),
    AppIcon(String),
    Summary(RuleTypeString),
    Body(RuleTypeString),
    Urgency(String),
    ExpireTimeout(i32)
}

impl TryFrom<&str> for Rule {

    type Error = ();
    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let parts = line.split_whitespace().collect::<Vec<&str>>();

        if parts.len() < 3 {
            debug!("Missing parameter");
            return Err(());
        }

        let name = match parts.get(0) {
            Some(s) => *s,
            _ => unreachable!("Who did you even get here?")
        };

        let value = parts[2..].join(" ").trim().to_owned();

        match name.trim() {
            "app_name" => Ok(Rule::AppName(value)),
            "app_icon" => Ok(Rule::AppIcon(value)),
            "summary" => {
                match parts[1] {
                    "=" => Ok(Rule::Summary(RuleTypeString::Literal(value))),
                    "match" => Ok(Rule::Summary(RuleTypeString::Regex(Regex::new(&value[..]).or(Err(()))?))),
                    _ => Err(())
                }
            },
            "body" => {
                match parts[1] {
                    "=" => Ok(Rule::Body(RuleTypeString::Literal(value))),
                    "match" => Ok(Rule::Body(RuleTypeString::Regex(Regex::new(&value[..]).or(Err(()))?))),
                    _ => Err(())
                }
            }
            "urgency" => Ok(Rule::Urgency(value)),
            "expire_timeout" => Ok(Rule::ExpireTimeout(value.parse().or(Err(()))?)),
            n => {
                debug!("Unknown property {}", n);
                Err(())
            }
        }
    }

}

impl Rule {

    fn is_match (&self, other: &Notification) -> bool {
        match self {
            Rule::AppIcon(v) => v == &other.app_icon,
            Rule::AppName(v) => v == &other.app_name,
            Rule::Summary(RuleTypeString::Literal(v)) => v == &other.summary,
            Rule::Summary(RuleTypeString::Regex(v)) => v.is_match(&other.summary),
            Rule::Body(RuleTypeString::Literal(v)) => v == &other.body,
            Rule::Body(RuleTypeString::Regex(v)) => v.is_match(&other.body),
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

#[derive(Debug)]
pub enum RuleTypeString {
    Literal(String),
    Regex(Regex)
}

impl PartialEq for RuleTypeString {

    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(s), Self::Literal(o)) => s == o,
            (Self::Regex(s), Self::Regex(o)) => s.as_str() == o.as_str(),
            _ => false
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

    use super::*;

    mod parse {
        use super::*;

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
            (RuleTypeString $fn_name:ident $cnf_key:literal = $cnf_value:literal $type:tt) => {
                #[test]
                fn $fn_name() {
                    let rule = Rule::try_from(&format!("{} = {}", $cnf_key, $cnf_value)[..]);
                    assert!(rule.is_ok());
                    let rule = rule.unwrap();
                    assert_eq!(Rule::$type(RuleTypeString::Literal($cnf_value.to_owned())), rule);
                    
                    let rule = Rule::try_from(&format!("{} match {}", $cnf_key, "[a-z]")[..]);
                    assert!(rule.is_ok());
                    let rule = rule.unwrap();
                    assert_eq!(Rule::$type(RuleTypeString::Regex(Regex::new("[a-z]").unwrap())), rule)
                }
            }
        }

        rule_try_from_macro!(rule_try_from_app_name "app_name" = "test_app_name" AppName);
        rule_try_from_macro!(rule_try_from_app_icon "app_icon" = "icon" AppIcon);
        rule_try_from_macro!(RuleTypeString rule_try_from_summary "summary" = "summ" Summary);
        rule_try_from_macro!(RuleTypeString rule_try_from_body "body" = "test body" Body);
        rule_try_from_macro!(rule_try_from_urgency "urgency" = "low" Urgency);
        rule_try_from_macro!(rule_try_from_expire_timeout "expire_timeout" = 100 ExpireTimeout);
    }

    mod rule_match {

        use super::*;

        fn new_notification() -> Notification {
            Notification {
                app_name: "".to_owned(),
                actions: vec![],
                app_icon: "".to_owned(),
                body: "".to_owned(),
                expire_timeout: 0,
                id: 0,
                summary: "".to_owned(),
                urgency: notify_server::notification::Urgency::Normal
            }
        }

        #[test]
        fn app_icon() {
            let rule = Rule::AppIcon(String::from("#"));
            let mut n = new_notification();
            n.app_icon = String::from("#");
            assert!(rule.is_match(&n));
            n.app_icon = String::new();
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn app_name() {
            let rule = Rule::AppName(String::from("name"));
            let mut n = new_notification();
            n.app_name = String::from("name");
            assert!(rule.is_match(&n));
            n.app_name = String::from("other");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn summary_literal() {
            let rule = Rule::Summary(RuleTypeString::Literal(String::from("summary")));
            let mut n = new_notification();
            n.summary = String::from("summary");
            assert!(rule.is_match(&n));
            n.summary = String::from("other");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn summary_regex() {
            let rule = Rule::Summary(RuleTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.summary = String::from("summary");
            assert!(rule.is_match(&n));
            n.summary = String::from("o2ther");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn body_literal() {
            let rule = Rule::Body(RuleTypeString::Literal(String::from("body")));
            let mut n = new_notification();
            n.body = String::from("body");
            assert!(rule.is_match(&n));
            n.body = String::from("other");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn body_regex() {
            let rule = Rule::Body(RuleTypeString::Regex(Regex::new("^[a-z]+$").unwrap()));
            let mut n = new_notification();
            n.body = String::from("body");
            assert!(rule.is_match(&n));
            n.body = String::from("bo2dy");
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn urgency() {
            let rule = Rule::Urgency("low".to_owned());
            let mut n = new_notification();
            n.urgency = notify_server::notification::Urgency::Low;
            assert!(rule.is_match(&n));
            n.urgency = notify_server::notification::Urgency::Critical;
            assert!(!rule.is_match(&n));
        }

        #[test]
        fn expire_timeout() {
            let rule = Rule::ExpireTimeout(42);
            let mut n = new_notification();
            n.expire_timeout = 42;
            assert!(rule.is_match(&n));
            n.expire_timeout = 21;
            assert!(!rule.is_match(&n));
        }

    }
}