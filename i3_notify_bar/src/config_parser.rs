use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::BufRead;
use std::str::FromStr;

use log::info;
use pest::error::LineColLocation;
use pest::{iterators::Pair, Parser};
use regex::Regex;

use crate::emoji::EmojiMode;
use crate::{
    icons,
    rule::{Action, ConditionTypeString, Conditions as Condition, Definition, SetProperty, Style},
    template,
};

#[derive(Parser)]
#[grammar = "config.pest"]
struct ConfigParser;

pub fn parse_config(config: &mut dyn BufRead) -> ParseResult<Vec<Definition>> {
    info!("Reading conditions");
    let config = config
        .lines()
        .map(Result::unwrap)
        .map(|mut line| {
            line.push('\n');
            line
        })
        .collect::<String>();

    let config = ConfigParser::parse(Rule::config, &config);
    let config = match config {
        Ok(config) => config,
        Err(e) => return Err(ParseError::from(e.line_col)),
    }
    .next()
    .unwrap();

    let definitions = config.into_inner().filter(|def| match def.as_rule() {
        Rule::definition => true,
        Rule::EOI => false,
        rule => panic!("Unexpected rule: {:#?}", rule),
    });

    Ok(definitions.map(parse_definition).collect::<Vec<_>>())
}

fn parse_definition(definition: Pair<Rule>) -> Definition {
    let mut def = Definition::default();
    let groups = definition.into_inner();
    groups.for_each(|section| {
        let section = section.into_inner().next().unwrap();
        match section.as_rule() {
            Rule::condition_section => parse_condition_section(section, &mut def.conditions),
            Rule::style_section => def.style = parse_style_section(section),
            Rule::action_section => def.actions = parse_action_section(section),
            _ => panic!(),
        }
    });
    def
}

fn parse_action_section(action_section: Pair<Rule>) -> Vec<Action> {
    action_section
        .into_inner()
        .map(parse_action)
        .collect::<Vec<_>>()
}

fn parse_action(action: Pair<Rule>) -> Action {
    let action = action.into_inner().next().unwrap();
    match action.as_rule() {
        Rule::set_action => parse_set_action(action),
        Rule::stop_action => Action::Stop,
        Rule::ignore_action => Action::Ignore,
        _ => panic!(),
    }
}

fn parse_set_action(set_action: Pair<Rule>) -> Action {
    let mut inner = set_action.into_inner();
    let key = inner.next().unwrap().into_inner().next().unwrap();
    let value = inner.next().unwrap().as_str();

    match key.as_rule() {
        Rule::app_icon => Action::Set(SetProperty::Icon(icons::get_icon(value).unwrap_or('\u{0}'))),
        Rule::message_id => Action::Set(SetProperty::Id(value.to_owned())),
        Rule::text => Action::Set(SetProperty::Text(
            template::add_template(value.to_owned()).unwrap(),
        )),
        Rule::expire_timeout => Action::Set(SetProperty::ExpireTimeout(value.parse().unwrap())),
        Rule::emoji_mode => {
            Action::Set(SetProperty::EmojiMode(EmojiMode::from_str(value).unwrap()))
        }
        _ => panic!(),
    }
}

fn parse_condition_section(condition_section: Pair<Rule>, conditions: &mut Vec<Condition>) {
    condition_section
        .into_inner()
        .filter(|condition| matches!(condition.as_rule(), Rule::condition))
        .map(parse_condition)
        .fold(conditions, |list, condition| {
            list.push(condition);
            list
        });
}

fn parse_condition(condition: Pair<Rule>) -> Condition {
    let condition = condition.into_inner().next().unwrap();
    match condition.as_rule() {
        Rule::number_condition => parse_number_condition(condition),
        Rule::string_condition => parse_string_condition(condition),
        Rule::legacy_condition => parse_legacy_condition(condition),
        _ => unimplemented!(),
    }
}

fn parse_number_condition(number_condition: Pair<Rule>) -> Condition {
    let mut inner = number_condition.into_inner();
    let name = inner.next().unwrap().as_str();
    let _ = inner.next().unwrap().as_str();
    let value = inner.next().unwrap().as_str().parse().unwrap();

    match name {
        "expire_timeout" => Condition::ExpireTimeout(value),
        _ => unimplemented!(),
    }
}

fn parse_string_condition(string_condition: Pair<Rule>) -> Condition {
    let mut inner = string_condition.into_inner();
    let name = inner.next().unwrap().into_inner().next().unwrap().as_rule();
    let eq = inner.next().unwrap().into_inner().next().unwrap().as_rule();
    let value = inner.next().unwrap().as_str();

    let condition_type = match eq {
        Rule::compare_eq => ConditionTypeString::Literal(value.to_owned()),
        Rule::compare_match => ConditionTypeString::Regex(Regex::new(value).unwrap()),
        _ => panic!(),
    };

    match name {
        Rule::summary => Condition::Summary(condition_type),
        Rule::body => Condition::Body(condition_type),
        _ => panic!(),
    }
}

fn parse_legacy_condition(legacy_condition: Pair<Rule>) -> Condition {
    let mut inner = legacy_condition.into_inner();
    let name = inner.next().unwrap().into_inner().next().unwrap().as_rule();
    let mut inner = inner.skip(1);
    let value = inner.next().unwrap().as_str().to_owned();

    match name {
        Rule::app_icon => Condition::AppIcon(value),
        Rule::app_name => Condition::AppName(value),
        Rule::urgency => Condition::Urgency(value),
        _ => panic!(),
    }
}

fn parse_style_section(style_section: Pair<Rule>) -> Vec<Style> {
    style_section
        .into_inner()
        .map(parse_style)
        .collect::<Vec<_>>()
}

fn parse_style(style: Pair<Rule>) -> Style {
    let style = style.into_inner().next().unwrap();
    match style.as_rule() {
        Rule::background_style => {
            let color = style.into_inner().next().unwrap().as_str().to_owned();
            Style::Background(color)
        }
        Rule::text_style => {
            let color = style.into_inner().next().unwrap().as_str().to_owned();
            Style::Text(color)
        }
        _ => panic!(),
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    line_col_location: LineColLocation,
}

impl From<LineColLocation> for ParseError {
    fn from(loc: LineColLocation) -> Self {
        Self {
            line_col_location: loc,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.line_col_location)
    }
}

impl Error for ParseError {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_style_section_single_style() {
        let style_section = ConfigParser::parse(
            Rule::style_section,
            r#"style
        background #fff 
        endstyle"#,
        )
        .unwrap()
        .next()
        .unwrap();
        let styles = parse_style_section(style_section);
        assert_eq!(styles, vec![Style::Background(String::from("#fff"))]);
    }

    #[test]
    fn parse_string_condition_app_name() {
        let condition = ConfigParser::parse(Rule::legacy_condition, "app_name = test")
            .unwrap()
            .next()
            .unwrap();
        let condition = parse_legacy_condition(condition);
        assert_eq!(condition, Condition::AppName("test".to_owned()));
    }

    #[test]
    fn parse_number_condition_expire_timeout() {
        let condition = ConfigParser::parse(Rule::number_condition, "expire_timeout = 42")
            .unwrap()
            .next()
            .unwrap();
        let condition = parse_number_condition(condition);
        assert_eq!(condition, Condition::ExpireTimeout(42));
    }

    #[test]
    fn parse_single_definition() {
        let definition = ConfigParser::parse(
            Rule::definition,
            r#"def
            action
                stop
                ignore
            endaction
            enddef"#,
        );

        assert!(definition.is_ok(), "{:#?}", definition);

        let definition = definition.unwrap().next().unwrap();

        let definition = parse_definition(definition);
        assert_eq!(
            definition,
            Definition {
                actions: vec![Action::Stop, Action::Ignore],
                ..Default::default()
            }
        )
    }

    #[test]
    fn parse_condition_section_conditions() {
        let condition_section = ConfigParser::parse(
            Rule::condition_section,
            r#"rule
            app_name = Thunderbird
            expire_timeout = 10
            body match new
        endrule"#,
        )
        .unwrap()
        .next()
        .unwrap();
        let mut conditions = Vec::new();
        parse_condition_section(condition_section, &mut conditions);
        assert_eq!(
            conditions,
            vec![
                Condition::AppName(String::from("Thunderbird")),
                Condition::ExpireTimeout(10),
                Condition::Body(ConditionTypeString::Regex(Regex::new("new").unwrap()))
            ]
        );
    }

    #[test]
    fn parse_action_section_multiple_actions() {
        let action_section = ConfigParser::parse(
            Rule::action_section,
            r#"action
            set text Hello World
            stop
            ignore
        endaction"#,
        )
        .unwrap()
        .next()
        .unwrap();
        let actions = parse_action_section(action_section);
        assert_eq!(actions.len(), 3);
        match actions[0] {
            // Comparing ids could break the test based on test order
            Action::Set(SetProperty::Text(_)) => assert!(true),
            _ => assert!(false),
        }
        assert_eq!(actions[1], Action::Stop);
        assert_eq!(actions[2], Action::Ignore);
    }

    #[test]
    fn parse_simple_config() {
        let config = r#"def
            rule
                app_name = Thunderbird
            endrule
            action
                set expire_timeout -1
            endaction
            style
                background #ff00ff
            endstyle
        enddef"#;
        let config = parse_config(&mut config.as_bytes());
        assert_eq!(
            config.unwrap(),
            vec![Definition {
                conditions: vec![Condition::AppName("Thunderbird".to_owned())],
                actions: vec![Action::Set(SetProperty::ExpireTimeout(-1))],
                style: vec![Style::Background("#ff00ff".to_owned())]
            }]
        )
    }

    #[test]
    fn parse_empty_config() {
        let config = "   \n ";
        let config = parse_config(&mut config.as_bytes()).unwrap();
        assert_eq!(config, vec![])
    }

    #[test]
    fn parse_multi_entry_config() {
        let config = r#"def
    rule
        app_name = Thunderbird
    endrule
enddef
def
    action
        ignore
    endaction
enddef
def
    style
        background #ff00ff
    endstyle
enddef"#;
        let config = parse_config(&mut config.as_bytes()).unwrap();
        assert_eq!(
            config,
            vec![
                Definition {
                    conditions: vec![Condition::AppName("Thunderbird".to_owned())],
                    ..Default::default()
                },
                Definition {
                    actions: vec![Action::Ignore],
                    ..Default::default()
                },
                Definition {
                    style: vec![Style::Background("#ff00ff".to_owned())],
                    ..Default::default()
                }
            ]
        );
    }
}

#[cfg(test)]
mod pest_tests {

    use pest::Parser;

    use super::*;

    macro_rules! rule_test {
        ($rule: ident, $key: literal $compare: literal $value: literal) => {
            let line = format!("{} {} {}\n", $key, $compare, $value);
            let parsed_rule = ConfigParser::parse(Rule::condition, &line);
            assert!(parsed_rule.is_ok());
            let parsed_rule = parsed_rule.unwrap().next();
            assert!(parsed_rule.is_some());
            let pair = parsed_rule.unwrap().into_inner().next().unwrap();

            let mut inner = pair.into_inner();

            let rule_key = inner.next();
            assert!(rule_key.is_some());
            assert_eq!(rule_key.unwrap().as_str(), $key);

            let compare = inner.next();
            assert!(compare.is_some());
            assert_eq!(compare.unwrap().as_str(), $compare);

            let eol = inner.next();
            assert!(eol.is_some());
            assert_eq!(eol.unwrap().as_str(), $value);
        };
    }

    #[test]
    fn rule_app_name() {
        rule_test!(app_name, "app_name" "=" "test app name");
    }

    #[test]
    fn rule_app_icon() {
        rule_test!(app_icon, "app_icon" "=" "test icon");
    }

    #[test]
    fn rule_summary() {
        rule_test!(summary, "summary" "match" "regex expr");
    }

    #[test]
    fn rule_body() {
        rule_test!(body, "body" "=" "test body");
    }

    #[test]
    fn rule_urgency() {
        rule_test!(urgency, "urgency" "=" "test urgency");
    }

    #[test]
    fn rule_expire_timeout() {
        rule_test!(expire_timeout, "expire_timeout" "=" "test expire_timeouturgency");
    }

    #[test]
    fn rule_section() {
        let parsed = ConfigParser::parse(
            Rule::condition_section,
            r#"rule
            app_name = aname
            body match test value
            expire_timeout = 10
            endrule"#,
        );

        assert!(parsed.is_ok(), "{:#?}", parsed);
        let mut parsed = parsed.unwrap();

        let rule_section = parsed.next().unwrap();
        let mut rules = rule_section.into_inner();
        assert_eq!(rules.next().unwrap().as_str(), "app_name = aname");
        assert_eq!(rules.next().unwrap().as_str(), "body match test value");
        assert_eq!(rules.next().unwrap().as_str(), "expire_timeout = 10");
    }

    #[test]
    fn condition_section() {
        let parsed = ConfigParser::parse(
            Rule::condition_section,
            r#"condition
            app_name = aname
            body match test value
            expire_timeout = 10
            endcondition"#,
        );

        assert!(parsed.is_ok(), "{:#?}", parsed);
        let mut parsed = parsed.unwrap();

        let rule_section = parsed.next().unwrap();
        let mut rules = rule_section.into_inner();
        assert_eq!(rules.next().unwrap().as_str(), "app_name = aname");
        assert_eq!(rules.next().unwrap().as_str(), "body match test value");
        assert_eq!(rules.next().unwrap().as_str(), "expire_timeout = 10");
    }

    #[test]
    fn rule_section_space_in_closing_tag() {
        let parsed = ConfigParser::parse(
            Rule::condition_section,
            r#"rule
            app_name = aname
            body match test value
            expire_timeout = 10
            end rule"#,
        );

        assert!(parsed.is_err(), "{:#?}", parsed);
    }

    #[test]
    fn action_section() {
        let parsed = ConfigParser::parse(
            Rule::action_section,
            r#"action
            set text test
            stop
            ignore
        endaction
        "#,
        );
        assert!(parsed.is_ok());
        let mut parsed = parsed.unwrap();

        let action_section = parsed.next().unwrap();
        let mut actions = action_section.into_inner();
        assert_eq!(actions.next().unwrap().as_str(), "set text test");
        assert_eq!(actions.next().unwrap().as_str(), "stop");
        assert_eq!(actions.next().unwrap().as_str(), "ignore");
    }

    #[test]
    fn style_section() {
        let parsed = ConfigParser::parse(
            Rule::style_section,
            r#"style
            background #ff00ff
            text #234
        endstyle"#,
        );
        assert!(parsed.is_ok(), "{:#?}", parsed);
        let mut parsed = parsed.unwrap();

        let style_section = parsed.next().unwrap();
        let mut styles = style_section.into_inner();
        assert_eq!(styles.next().unwrap().as_str(), "background #ff00ff");
        assert_eq!(styles.next().unwrap().as_str(), "text #234");
    }

    #[test]
    fn definition() {
        let parsed = ConfigParser::parse(
            Rule::definition,
            r#"def
        style
            background #ff00ff
            text #234
        endstyle
        enddef"#,
        );

        assert!(parsed.is_ok(), "{:#?}", parsed);
        let mut parsed = parsed.unwrap();
        let definition = parsed.next().unwrap();

        let section = definition.into_inner().next().unwrap();
        let style_section = section.into_inner().next().unwrap();
        assert_eq!(style_section.as_rule(), Rule::style_section);
        let mut styles = style_section.into_inner();
        assert_eq!(styles.next().unwrap().as_str(), "background #ff00ff");
        assert_eq!(styles.next().unwrap().as_str(), "text #234");
    }

    #[test]
    fn config() {
        let parsed = ConfigParser::parse(
            Rule::config,
            r#"def
            style
                background #ff00ff
                text #234
            endstyle
            enddef"#,
        );

        assert!(parsed.is_ok(), "{:#?}", parsed);
        let mut parsed = parsed.unwrap();
        let config = parsed.next().unwrap();

        let definition = config.into_inner().next().unwrap();

        let section = definition.into_inner().next().unwrap();
        assert_eq!(section.as_rule(), Rule::section);
        let style_section = section.into_inner().next().unwrap();
        assert!(style_section.as_rule() == Rule::style_section);
        let mut styles = style_section.into_inner();
        assert_eq!(styles.next().unwrap().as_str(), "background #ff00ff");
        assert_eq!(styles.next().unwrap().as_str(), "text #234");
    }
}
