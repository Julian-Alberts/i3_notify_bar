use log::error;
use regex::Regex;

use super::{value::Value, Statement, StorageMethod, Template};

const STATEMENT_REGEX_STR: &str = r#"\{(?P<var_name>[a-z_0-9]+)(?P<modifier>|[^}]+)\}"#;
const MODIFIER_REGEX_STR: &str = r#"\|(?P<modifier>[a-z_0-9]+)(?P<args>:[^|]+)*"#;
const ARGS_REGEX_STR: &str =
    r#":(?P<var>[a-z][a-z_0-9]*)|(?:"(?P<str>(?:(?:\\")|[^"])*)")|(?P<num>[+-]?\d+(.\d+)?)"#;

pub fn compile(tpl: String) -> Template {
    let mut compiled_tpl = Template {
        tpl_str: tpl,
        tpl: Vec::new(),
    };

    let tpl = &compiled_tpl.tpl_str[..];
    // Compiling the regex should not fail
    let statement_regex = Regex::new(STATEMENT_REGEX_STR).unwrap();
    let modifier_regex = Regex::new(MODIFIER_REGEX_STR).unwrap();
    let args_regex = Regex::new(ARGS_REGEX_STR).unwrap();

    let mut end_of_last_match = 0;
    for captures in statement_regex.captures_iter(tpl) {
        //If there was no full match the iterrator would not run
        let full_match = captures.get(0).unwrap();
        if end_of_last_match != full_match.start() {
            let v = &tpl[end_of_last_match..full_match.start()];
            compiled_tpl.tpl.push(Statement::Literal(v));
        }
        end_of_last_match = full_match.end();

        let var_name = &captures["var_name"];
        let modifiers;

        if let Some(modifier) = captures.name("modifier") {
            modifiers = modifier_regex
                .captures_iter(modifier.as_str())
                .map(|m| {
                    let modifier_name = &m["modifier"];
                    let args_list;
                    if let Some(args) = m.name("args") {
                        args_list = args_regex
                            .captures_iter(args.as_str())
                            .map(|a| {
                                match (a.name("num"), a.name("str"), a.name("var")) {
                                    (Some(num), None, None) => {
                                        // If the conversion from string to f64 fails the regex does not work and should be checked
                                        StorageMethod::Const(Value::Number(
                                            num.as_str().parse().unwrap(),
                                        ))
                                    }
                                    (None, Some(string), None) => StorageMethod::Const(
                                        Value::String(string.as_str().to_string()),
                                    ),
                                    (None, None, Some(var)) => {
                                        StorageMethod::Variable(var.as_str())
                                    }
                                    _ => {
                                        error!(
                                            "{} did match {} but did not match any groups",
                                            args.as_str(),
                                            ARGS_REGEX_STR
                                        );
                                        unreachable!(
                                            "{} did match {} but did not match any groups",
                                            args.as_str(),
                                            ARGS_REGEX_STR
                                        );
                                    }
                                }
                            })
                            .collect();
                    } else {
                        args_list = Vec::new();
                    }
                    (modifier_name as *const _, args_list)
                })
                .collect();
        } else {
            modifiers = Vec::with_capacity(0);
        }

        compiled_tpl.tpl.push(Statement::Calculated {
            var_name,
            modifiers,
        })
    }

    if end_of_last_match < tpl.len() {
        compiled_tpl
            .tpl
            .push(Statement::Literal(&tpl[end_of_last_match..]));
    }

    compiled_tpl
}

#[cfg(test)]
/// This code in only used in tests
/// TODO clean up
impl PartialEq for Statement {
    fn eq(&self, other: &Statement) -> bool {
        match (self, other) {
            (Statement::Literal(s), Statement::Literal(o)) => unsafe { s.as_ref() == o.as_ref() },
            (
                Statement::Calculated {
                    var_name: s_var_name,
                    modifiers: s_modifiers,
                },
                Statement::Calculated {
                    var_name: o_var_name,
                    modifiers: o_modifiers,
                },
            ) => {
                unsafe {
                    if s_var_name.as_ref().unwrap() != o_var_name.as_ref().unwrap() {
                        return false;
                    }
                }

                if s_modifiers.len() != o_modifiers.len() {
                    return false;
                }

                if s_modifiers.iter().zip(o_modifiers).any(|(sp, op)| {
                    unsafe {
                        if sp.0.as_ref() != op.0.as_ref() {
                            return true;
                        }
                    }

                    if sp.1.iter().zip(op.1.iter()).any(|(s, o)| {
                        match (s, o) {
                            (StorageMethod::Const(s), StorageMethod::Const(o)) => {
                                if s != o {
                                    return true;
                                }
                            }
                            (StorageMethod::Variable(s), StorageMethod::Variable(o)) => unsafe {
                                if s.as_ref() != o.as_ref() {
                                    return true;
                                }
                            },
                            _ => return true,
                        }
                        false
                    }) {
                        return true;
                    }
                    false
                }) {
                    return false;
                }

                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod regex {

        use super::*;

        #[test]
        fn test_var() {
            let regex = Regex::new(STATEMENT_REGEX_STR).unwrap();
            let expected = vec![("foo", 7, 12), ("bar", 22, 27)];
            let input = "Simple {foo} template {bar} string";

            regex
                .captures_iter(input)
                .zip(regex.find_iter(input))
                .zip(expected.into_iter())
                .for_each(|((a, area), (e, start, end))| {
                    assert_eq!(&a["var_name"], e);
                    assert_eq!(area.start(), start);
                    assert_eq!(area.end(), end);
                });
        }

        #[test]
        fn test_var_modifier() {
            let regex = Regex::new(STATEMENT_REGEX_STR).unwrap();
            let expected = vec![("foo", "|qwerty"), ("bar", "|foobar:arg1:arg2|barfoo")];
            let input = "Simple {foo|qwerty} template {bar|foobar:arg1:arg2|barfoo} string";

            regex
                .captures_iter(input)
                .zip(expected.into_iter())
                .for_each(|(a, (v, m))| {
                    assert_eq!(&a["var_name"], v);
                    assert_eq!(&a["modifier"], m);
                });
        }

        #[test]
        fn test_var_modifier_args() {
            let regex = Regex::new(MODIFIER_REGEX_STR).unwrap();
            let expected = vec![("foobar", ":arg1:arg2"), ("barfoo", ":asd")];
            let input = "|foobar:arg1:arg2|barfoo:asd";

            regex
                .captures_iter(input)
                .zip(expected.into_iter())
                .for_each(|(a, (m, args))| {
                    assert_eq!(&a["modifier"], m);
                    assert_eq!(&a["args"], args);
                });
        }

        #[test]
        fn test_var_modifier_arg_types() {
            let regex = Regex::new(ARGS_REGEX_STR).unwrap();
            let input = r#":var_name:"string_value":42.42"#;
            let expected = vec![
                ("var", "var_name"),
                ("str", "string_value"),
                ("num", "42.42"),
            ];

            regex
                .captures_iter(input)
                .zip(expected.into_iter())
                .for_each(|(arg, expected)| {
                    if let Some(arg) = arg.name(expected.0) {
                        assert_eq!(arg.as_str(), expected.1);
                    } else {
                        panic!("{:#?}", arg);
                    }
                });
        }
    }

    #[test]
    fn simple_compile() {
        let tpl = compile("Simple template string".to_owned());
        assert_eq!(
            vec![Statement::Literal("Simple template string" as *const _)],
            tpl.tpl
        );
    }

    #[test]
    fn variable_value() {
        let tpl = compile("Simple {var} template {foo}".to_owned());
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    var_name: "var" as *const _,
                    modifiers: vec![]
                },
                Statement::Literal(" template " as *const _),
                Statement::Calculated {
                    var_name: "foo",
                    modifiers: vec![]
                }
            ],
            tpl.tpl
        )
    }

    #[test]
    fn variable_value_simple_modifier() {
        let tpl = compile("Simple {var|test} template".to_owned());
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    var_name: "var" as *const _,
                    modifiers: vec![("test" as *const _, vec![])]
                },
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }

    #[test]
    fn variable_value_modifier_string_value() {
        let tpl = compile(r#"Simple {var|test:"test value"} template"#.to_owned());
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    var_name: "var" as *const _,
                    modifiers: vec![(
                        "test" as *const _,
                        vec![StorageMethod::Const(Value::String(
                            "test value".to_string()
                        ))]
                    )]
                },
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }

    #[test]
    fn variable_value_modifier_num_value() {
        let tpl = compile(r#"Simple {var|test:42} template"#.to_owned());
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    var_name: "var" as *const _,
                    modifiers: vec![(
                        "test" as *const _,
                        vec![StorageMethod::Const(Value::Number(42_f64))]
                    )]
                },
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }

    #[test]
    fn variable_value_modifier_var_value() {
        let tpl = compile(r#"Simple {var|test:foobar} template"#.to_owned());
        assert_eq!(
            vec![
                Statement::Literal("Simple " as *const _),
                Statement::Calculated {
                    var_name: "var" as *const _,
                    modifiers: vec![("test" as *const _, vec![StorageMethod::Variable("foobar")])]
                },
                Statement::Literal(" template" as *const _)
            ],
            tpl.tpl
        );
    }
}
