use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::BufRead;

use log::{error, info};
use pest::{iterators::Pair, Parser};

#[derive(Debug, Default, PartialEq)]
pub struct ConfigDef {
    pub rules: Vec<RuleDef>,
}

#[derive(Debug, Default, PartialEq)]
pub struct RuleDef {
    pub conditions: Vec<ConditionDef>,
    pub actions: Vec<ActionDef>,
    pub style: Vec<StyleDef>,
    pub sub_rules: Vec<RuleDef>,
}

#[derive(Debug, PartialEq)]
pub struct ConditionDef {
    pub property: PropertyName,
    pub op: CompareOperation,
    pub value: Value,
}

#[derive(Debug, Default, PartialEq)]
pub struct PropertyName(pub String);
#[derive(Debug, Default, PartialEq)]
pub struct Value(pub String);

#[derive(Debug, PartialEq)]
pub enum CompareOperation {
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    Match,
}

#[derive(Debug, PartialEq)]
pub enum ActionDef {
    Set(ActionSetDef),
    Stop,
    Ignore,
}

#[derive(Debug, PartialEq)]
pub struct ActionSetDef {
    pub property: PropertyName,
    pub value: Value,
}

#[derive(Debug, Default, PartialEq)]
pub struct StyleDef {
    pub property: PropertyName,
    pub value: Value,
}

#[derive(Parser)]
#[grammar = "config.pest"]
pub struct ConfigParser;

pub fn parse_config(config: &mut dyn BufRead) -> ParseResult<ConfigDef> {
    info!("Reading conditions");
    let config = config
        .lines()
        .map(unwrap_line)
        .map(|mut line| {
            line.push('\n');
            line
        })
        .collect::<String>();

    let config = ConfigParser::parse(Rule::config, &config);
    let config = match config {
        Ok(config) => config,
        Err(e) => return Err(ParseError::PestError(Box::new(e))),
    }
    .next();

    let config = match config {
        Some(c) => c,
        None => return Err(ParseError::UnexpectedEnd),
    };

    let rules = config.into_inner().filter(|def| match def.as_rule() {
        Rule::rule => true,
        Rule::EOI => false,
        other => unreachable!("Unexpected rule: {:#?}", other),
    });

    let rules = rules.map(parse_rule).collect::<Result<_, _>>()?;
    Ok(ConfigDef { rules })
}

fn unwrap_line(result: Result<String, std::io::Error>) -> String {
    match result {
        Ok(r) => r,
        Err(e) => {
            error!(r#"Could not read line "{}""#, e);
            panic!(r#"Could not read line "{}""#, e)
        }
    }
}

fn parse_rule(definition: Pair<Rule>) -> ParseResult<RuleDef> {
    let mut rule = RuleDef::default();
    let groups = definition.into_inner();
    for section in groups {
        let section = section
            .into_inner()
            .next()
            .ok_or(ParseError::UnexpectedEnd)?;
        match section.as_rule() {
            Rule::condition_section => parse_condition_section(section, &mut rule.conditions)?,
            Rule::style_section => parse_style_section(section, &mut rule.style)?,
            Rule::action_section => parse_action_section(section, &mut rule.actions)?,
            Rule::rule => rule.sub_rules.push(parse_rule(section)?),
            _ => unreachable!(),
        }
    }
    Ok(rule)
}

fn parse_action_section(
    action_section: Pair<Rule>,
    actions: &mut Vec<ActionDef>,
) -> ParseResult<()> {
    action_section
        .into_inner()
        .map(parse_action)
        .try_fold(actions, try_collect)?;
    Ok(())
}

fn parse_action(action: Pair<Rule>) -> ParseResult<ActionDef> {
    let action = action
        .into_inner()
        .next()
        .ok_or(ParseError::UnexpectedEnd)?;
    match action.as_rule() {
        Rule::set_action => parse_set_action(action).map(ActionDef::Set),
        Rule::stop_action => Ok(ActionDef::Stop),
        Rule::ignore_action => Ok(ActionDef::Ignore),
        _ => unreachable!(),
    }
}

fn parse_set_action(set_action: Pair<Rule>) -> ParseResult<ActionSetDef> {
    let mut inner = set_action.into_inner();
    let key = inner.next().ok_or(ParseError::UnexpectedEnd)?;
    let property = parse_property(key);
    let value = parse_value(inner.next().ok_or(ParseError::UnexpectedEnd)?);

    Ok(ActionSetDef { property, value })
}

fn parse_condition_section(
    condition_section: Pair<Rule>,
    conditions: &mut Vec<ConditionDef>,
) -> ParseResult<()> {
    let mut new_conditions = condition_section
        .into_inner()
        .filter(|condition| matches!(condition.as_rule(), Rule::condition))
        .map(parse_condition)
        .collect::<ParseResult<Vec<_>>>()?;

    conditions.append(&mut new_conditions);
    Ok(())
}

fn parse_condition(condition: Pair<Rule>) -> ParseResult<ConditionDef> {
    let mut condition_iter = condition.into_inner();
    let property_name = condition_iter.next().ok_or(ParseError::UnexpectedEnd)?;
    let property = parse_property(property_name);

    let op = condition_iter.next().ok_or(ParseError::UnexpectedEnd)?;
    let op = parse_compare_op(op);
    let value = condition_iter.next().ok_or(ParseError::UnexpectedEnd)?;
    let value = parse_value(value);
    Ok(ConditionDef {
        property,
        op,
        value,
    })
}

fn parse_value(value: Pair<'_, Rule>) -> Value {
    if value.as_rule() != Rule::assign_value {
        unreachable!()
    }
    Value(value.as_str().to_string())
}

fn parse_compare_op(op: Pair<'_, Rule>) -> CompareOperation {
    if op.as_rule() != Rule::compare_op {
        unreachable!()
    }
    match op
        .into_inner()
        .next()
        .expect("Compare is missing operator")
        .as_rule()
    {
        Rule::compare_eq => CompareOperation::Eq,
        Rule::compare_lt => CompareOperation::Lt,
        Rule::compare_le => CompareOperation::Le,
        Rule::compare_gt => CompareOperation::Gt,
        Rule::compare_ge => CompareOperation::Ge,
        Rule::compare_match => CompareOperation::Match,
        _ => unreachable!(),
    }
}

fn parse_property(property: Pair<Rule>) -> PropertyName {
    if property.as_rule() == Rule::property {
        PropertyName(property.as_str().to_owned())
    } else {
        unreachable!()
    }
}

fn parse_style_section(style_section: Pair<Rule>, styles: &mut Vec<StyleDef>) -> ParseResult<()> {
    style_section
        .into_inner()
        .map(parse_style)
        .try_fold(styles, try_collect)?;
    Ok(())
}

fn parse_style(style: Pair<Rule>) -> ParseResult<StyleDef> {
    let mut token_iter = style.into_inner();
    let property = parse_property(token_iter.next().ok_or(ParseError::UnexpectedEnd)?);

    let color = parse_value(token_iter.next().ok_or(ParseError::UnexpectedEnd)?);
    Ok(StyleDef {
        property,
        value: color,
    })
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    PestError(Box<pest::error::Error<Rule>>),
    UnexpectedEnd,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let e: &dyn Error = match self {
            Self::PestError(e) => e,
            Self::UnexpectedEnd => return write!(f, "Unexpected end of config file"),
        };

        Display::fmt(e, f)
    }
}

impl Error for ParseError {}

fn try_collect<T, E>(c: &mut Vec<T>, v: Result<T, E>) -> Result<&mut Vec<T>, E> {
    c.push(v?);
    Ok(c)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[should_panic(expected = "Could not read line \"Test Error\"")]
    fn unwrap_line_error() {
        unwrap_line(Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Test Error",
        )));
    }

    #[test]
    fn parse_style_section_single_style() {
        let style_section = ConfigParser::parse(
            Rule::style_section,
            r#"style
        background #fff 
        end"#,
        )
        .unwrap()
        .next()
        .unwrap();
        let mut styles = Vec::new();
        parse_style_section(style_section, &mut styles).unwrap();
        assert_eq!(
            styles,
            vec![StyleDef {
                property: PropertyName("background".to_string()),
                value: Value("#fff".to_owned())
            }]
        );
    }

    #[test]
    fn parse_single_rule() {
        let definition = ConfigParser::parse(
            Rule::rule,
            r#"rule
            action
                stop
                ignore
            end
            end"#,
        );

        assert!(definition.is_ok(), "{:#?}", definition);

        let definition = definition.unwrap().next().unwrap();

        let definition = parse_rule(definition).unwrap();
        assert_eq!(
            definition,
            RuleDef {
                actions: vec![ActionDef::Stop, ActionDef::Ignore],
                ..Default::default()
            }
        )
    }

    #[test]
    fn parse_rule_with_sub_rule() {
        let definition = ConfigParser::parse(
            Rule::rule,
            r#"rule
                action
                    stop
                end
                rule
                    condition
                        app_name = TestApp
                    end
                    action
                        ignore
                    end
                end
            end"#,
        );

        assert!(definition.is_ok(), "{:#?}", definition);

        let definition = definition.unwrap().next().unwrap();

        let definition = parse_rule(definition).unwrap();
        assert_eq!(
            definition,
            RuleDef {
                actions: vec![ActionDef::Stop],
                sub_rules: vec![RuleDef {
                    conditions: vec![ConditionDef {
                        property: PropertyName("app_name".to_owned()),
                        op: CompareOperation::Eq,
                        value: Value("TestApp".to_string())
                    }],
                    actions: vec![ActionDef::Ignore],
                    ..Default::default()
                }],
                ..Default::default()
            }
        )
    }

    #[test]
    fn parse_condition_section_conditions() {
        let condition_section = ConfigParser::parse(
            Rule::condition_section,
            r#"condition
            app_name = Thunderbird
            expire_timeout = 10
            body match new
        end"#,
        )
        .unwrap()
        .next()
        .unwrap();
        let mut conditions = Vec::new();
        parse_condition_section(condition_section, &mut conditions).unwrap();
        assert_eq!(
            conditions,
            vec![
                ConditionDef {
                    property: PropertyName(String::from("app_name")),
                    op: CompareOperation::Eq,
                    value: Value("Thunderbird".to_owned())
                },
                ConditionDef {
                    property: PropertyName(String::from("expire_timeout")),
                    op: CompareOperation::Eq,
                    value: Value("10".to_owned())
                },
                ConditionDef {
                    property: PropertyName(String::from("body")),
                    op: CompareOperation::Match,
                    value: Value("new".to_owned())
                },
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
        end"#,
        )
        .unwrap()
        .next()
        .unwrap();
        let mut actions = Vec::new();
        parse_action_section(action_section, &mut actions).unwrap();
        assert_eq!(actions.len(), 3);

        assert_eq!(
            actions[0],
            ActionDef::Set(ActionSetDef {
                property: PropertyName("text".to_owned()),
                value: Value("Hello World".to_owned()),
            })
        );
        assert_eq!(actions[1], ActionDef::Stop);
        assert_eq!(actions[2], ActionDef::Ignore);
    }

    #[test]
    fn parse_simple_config() {
        let config = r#"rule
            condition
                app_name = Thunderbird
            end
            action
                set expire_timeout -1
            end
            style
                background #ff00ff
            end
        end"#;
        let config = parse_config(&mut config.as_bytes());
        assert_eq!(
            config.unwrap(),
            ConfigDef {
                rules: vec![RuleDef {
                    conditions: vec![ConditionDef {
                        property: PropertyName("app_name".to_owned()),
                        op: CompareOperation::Eq,
                        value: Value("Thunderbird".to_owned()),
                    }],
                    actions: vec![ActionDef::Set(ActionSetDef {
                        property: PropertyName("expire_timeout".to_string()),
                        value: Value("-1".to_owned())
                    })],
                    style: vec![StyleDef {
                        property: PropertyName("background".to_owned()),
                        value: Value("#ff00ff".to_owned()),
                    }],
                    sub_rules: vec![]
                }]
            }
        )
    }

    #[test]
    fn parse_empty_config() {
        let config = "   \n ";
        let config = parse_config(&mut config.as_bytes()).unwrap();
        assert_eq!(config, ConfigDef::default())
    }

    #[test]
    fn parse_multi_entry_config() {
        let config = r#"rule
    condition
        app_name = Thunderbird
    end
end
rule
    action
        ignore
    end
end
rule
    style
        background #ff00ff
    end
end"#;
        let config = parse_config(&mut config.as_bytes()).unwrap();
        assert_eq!(
            config,
            ConfigDef {
                rules: vec![
                    RuleDef {
                        conditions: vec![ConditionDef {
                            property: PropertyName("app_name".to_owned()),
                            op: CompareOperation::Eq,
                            value: Value("Thunderbird".to_owned())
                        }],
                        ..Default::default()
                    },
                    RuleDef {
                        actions: vec![ActionDef::Ignore],
                        ..Default::default()
                    },
                    RuleDef {
                        style: vec![StyleDef {
                            property: PropertyName("background".to_owned()),
                            value: Value("#ff00ff".to_owned())
                        }],
                        ..Default::default()
                    }
                ]
            }
        );
    }
}

#[cfg(test)]
mod pest_tests {

    use pest::Parser;

    use super::*;

    #[test]
    fn rule_section() {
        let parsed = ConfigParser::parse(
            Rule::condition_section,
            r#"condition
            app_name = aname
            body match test value
            expire_timeout = 10
            end"#,
        );

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
            end"#,
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
        end"#,
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
            Rule::rule,
            r#"rule
        style
            background #ff00ff
            text #234
        end
        end"#,
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
            r#"rule
            style
                background #ff00ff
                text #234
            end
            end"#,
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
