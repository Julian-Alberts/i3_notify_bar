use regex::Regex;

use super::{prelude::*, value::Value};

#[macro_export]
macro_rules! create_modifier {
    (|$first_name:ident: $first_t: ty $(,$name: ident: $t: ty)*| -> $return: ty $b: block) => {
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
                    None => Err("Missing arguments".to_owned())?
                };
            )*

            fn inner($first_name: $first_t $(,$name: $t)*) -> $return $b;

            let result = inner($first_name $(,$name)*);
            Ok(result.into())
        }
    };
}

pub type Modifier = dyn Fn(&Value, Vec<&Value>) -> Result<Value, String>;

pub const SLICE: &Modifier = &create_modifier!(|input: String, start: usize, end: usize| -> String {
    input[start..end].to_owned()
});

pub const MATCH: &Modifier = &create_modifier!(|input: String, regex: String| -> String {
    let regex = Regex::new(&regex).unwrap();
    let c = match regex.captures(&input[..]) {
        Some(c) => match c.get(0) {
            Some(c) => c.as_str(),
            None => ""
        },
        None => ""
    };

    c.to_owned()
    
});