use std::io::BufRead;
use std::io::{Error, ErrorKind};
use std::convert::TryFrom;

use super::rule::{Action, Definition, Rule, Style};

/*
    pub app_name: String,
    pub app_icon: String, 
    pub summary: String, 
    pub body: String,
    pub urgency: Urgency,
    pub actions: Vec<String>, 
    pub expire_timeout: i32


def
    rule
        app_name = match
        app_icon = match
        summary = match
        body = match
        urgency = one of
        expire_timeout = < >
    endrule
    action
        ignore
    endaction
enddef

*/

macro_rules! error {
    ($($arg:tt)*) => {
        Err(Error::new(ErrorKind::Other, format!($($arg)*)))
    };
}

pub fn parse_config(config: &mut dyn BufRead) -> std::io::Result<Vec<Definition>> {

    let mut definitions = Vec::new();
    let mut def = None;
    let mut rules = None;
    let mut actions = None;
    let mut styles = None;

    let line_iter = config.lines().enumerate();

    for (line_num, line) in line_iter {
        let line = line.unwrap();
        let line = line.trim();
        match (line, &mut def, &mut rules, &mut actions, &mut styles) {
            ("def", None, None, None, None) => 
                def = Some(Definition::default()),
            ("enddef", Some(_), None, None, None) => {
                definitions.push(def.unwrap());
                def = None
            },
            ("rule", Some(_), None, None, None) => 
                rules = Some(Vec::new()),
            ("endrule", Some(def), Some(_), None, None) => {
                def.rules = rules.unwrap();
                rules = None
            },
            ("action", Some(_), None, None, None) => 
                actions = Some(Vec::new()),
            ("endaction", Some(def), None, Some(_), None) => {
                def.actions = actions.unwrap();
                actions = None
            },
            ("style", Some(_), None, None, None) => 
                styles = Some(Vec::new()),
            ("endstyle", Some(def), None, None, Some(_)) => {
                def.style = styles.unwrap();
                styles = None
            }
            (rule_line, Some(_), Some(rules), None, None) => {
                let split = rule_line.splitn(2, '=');
                let split = split.collect::<Vec<&str>>();
                if split.len() != 2 {
                    return error!("Missing argument in line {}", line_num)
                }

                let name = split[0];
                let value = split[1];

                let r = rule_name_to_rule(name, value);
                match r {
                    Some(r) => rules.push(r),
                    None => return error!("Could not parse line {} \"{}\"", line_num, rule_line)
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
    
    Ok(definitions)
}

fn rule_name_to_rule(rule_name: &str, value: &str) -> Option<Rule> {
    match rule_name.trim() {
        "app_name" => Some(Rule::AppName(value.trim().to_owned())),
        "app_icon" => Some(Rule::AppIcon(value.trim().to_owned())),
        "summary" => Some(Rule::Summary(value.trim().to_owned())),
        "body" => Some(Rule::Body(value.trim().to_owned())),
        "urgency" => Some(Rule::Urgency(value.trim().to_owned())),
        "expire_timeout" => Some(Rule::ExpireTimeout(value.trim().parse().ok()?)),
        _ => None
    }
}