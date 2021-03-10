use log::error;
use regex::Regex;

use super::{prelude::*, value::Value};

#[macro_export]
macro_rules! create_modifier {
    (|$first_name:ident: $first_t: ty $(,$name: ident: $t: ty $(= $default: expr;)?)*| -> $return: ty $b: block) => {
        |value, args| {
            
            let $first_name: $first_t = match value.try_into() {
                Ok($first_name) => $first_name,
                Err(_) => Err("Can not parse value".to_owned())?
            };

            #[allow(unused_mut, unused_variables)]
            let mut args = args.into_iter();
            $(
                let $name: $t = match args.next() {
                    Some($name) => match $name.try_into() {
                        Ok($name) => $name,
                        Err(_) => Err("Can not parse value".to_owned())?
                    }
                    None => create_modifier!(default_value $($default)?)
                };
            )*

            fn inner($first_name: $first_t $(,$name: $t)*) -> $return $b;

            let result = inner($first_name $(,$name)*);
            Ok(result.into())
        }
    };
    (default_value) => {
        Err("Can not parse value".to_owned())?
    };
    (default_value $default: tt) => {
        $default
    }
}

pub type Modifier = dyn Fn(&Value, Vec<&Value>) -> Result<Value, String>;

pub const SLICE: &Modifier = &create_modifier!(|input: String, start: usize, end: usize| -> String {
    input[start..end].to_owned()
});

pub const MATCH: &Modifier = &create_modifier!(|input: String, regex: String, group: usize = 0;| -> String {
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

        let result = MATCH(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2 string"))));

        let args = vec![
            &regex,
        ];

        let result = MATCH(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2 string"))));

        let args = vec![
            &regex,
            &group
        ];
        let result = MATCH(&input, args);
        assert_eq!(result, Ok(Value::String(String::from("2test2"))));

        let args = vec![
            &invalid_regex,
            &full_match
        ];
        let result = MATCH(&input, args);
        assert_eq!(result, Ok(input))
    }

}