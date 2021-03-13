use log::error;
use regex::Regex;

use super::value::Value;
pub use error::*;

#[macro_export]
macro_rules! create_modifier {
    (fn $modifier_name: ident ($first_name:ident: $first_t: ty $($(,$name: ident: $t: ty $(= $default: expr)?)+)?) -> $return: ty $b: block) => {
        #[allow(unused_variables)]
        pub fn $modifier_name(value: &$crate::mini_template::value::Value, args: Vec<&$crate::mini_template::value::Value>) -> $crate::mini_template::modifier::error::Result<Value> {
            use $crate::mini_template::{modifier::error::ErrorKind, prelude::*};

            let $first_name: $first_t = match value.try_into() {
                Ok($first_name) => $first_name,
                Err(_) => Err(ErrorKind::TypeError{value: value.to_string(), expected_type: stringify!($first_t)})?
            };

            $(
                let mut args = args.into_iter();
                $(
                    let $name: $t = match args.next() {
                        Some($name) => match $name.try_into() {
                            Ok($name) => $name,
                            Err(_) => Err(ErrorKind::TypeError{value: $name.to_string(), expected_type: stringify!($t)})?
                        }
                        None => create_modifier!(default_value $name $($default)?)
                    };
                )+
            )?

            fn inner($first_name: $first_t $($(,$name: $t)+)?) -> $return $b;

            let result = inner($first_name $($(,$name)+)?);
            Ok(result.into())
        }
    };
    (default_value $arg_name: ident) => {
        Err(ErrorKind::MissingArgument{argument_name: stringify!($arg_name)})?
    };
    (default_value $arg_name: ident $default: tt) => {
        $default
    }
}

pub type Modifier = dyn Fn(&Value, Vec<&Value>) -> Result<Value>;

create_modifier!(fn slice_modifier(input: String, start: usize, end: usize) -> String {
    input[start..end].to_owned()
});

create_modifier!(fn match_modifier(input: String, regex: String, group: usize = 0) -> String {
    let regex = match Regex::new(&regex) {
        Ok(r) => r,
        Err(r) => {
            error!("{}", r.to_string());
            return input
        }
    };
    let c = match regex.captures(&input[..]) {
        Some(c) => match c.get(group) {
            Some(c) => c.as_str(),
            None => ""
        },
        None => ""
    };

    c.to_owned()
    
});

pub mod error {

    pub type Result<T> = std::result::Result<T, ErrorKind>;
    #[derive(Debug, PartialEq)]
    pub enum ErrorKind {
        MissingArgument{argument_name: &'static str},
        TypeError{value: String, expected_type: &'static str}
    }

    impl ToString for ErrorKind {

        fn to_string(&self) -> String {
            match self {
                Self::MissingArgument{argument_name} => format!("Missing argument \"{}\"", argument_name),
                Self::TypeError{value, expected_type} => format!("Can not convert {} to type {}", value, expected_type)
            }
        }

    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn match_modifier() {
        let input = Value::String(String::from("My 2test2 string"));
        let regex = Value::String(String::from(r#"(\d[a-z]+\d) string"#));
        let invalid_regex = Value::String(String::from(r#"(\d[a-z]+\d string"#));
        let full_match = Value::Number(0.0);
        let group = Value::Number(1.0);
        let args = vec![
            &regex,
            &full_match
        ];

        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2 string"))));

        let args = vec![
            &regex,
        ];

        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2 string"))));

        let args = vec![
            &regex,
            &group
        ];
        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2"))));

        let args = vec![
            &invalid_regex,
            &full_match
        ];
        let result = super::match_modifier(&input, args);
        assert_eq!(result, Ok(input))
    }

    #[test]
    fn missing_argument() {
        let input = Value::String(String::from("My test string"));
        let args = vec![];

        let result = super::match_modifier(&input, args);
        assert_eq!(result, Err(ErrorKind::MissingArgument{argument_name: "regex"}));
    }

    #[test]
    fn can_not_parse_argument() {
        let input = Value::String(String::from("My test string"));

        let regex = Value::String(String::from(r#"(\d[a-z]+\d) string"#));
        let number = Value::String(String::from("test"));

        let args = vec![
            &regex,
            &number
        ];

        let result = super::match_modifier(&input, args);
        assert_eq!(result, Err(ErrorKind::TypeError{expected_type: "usize", value: String::from("test")}));
    }
}