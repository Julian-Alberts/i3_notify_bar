use super::{prelude::*, value::Value};

macro_rules! create_filter {
    (|$($name: ident: $t: ty),+| -> $return: ty $b: block) => {
        |args| {
            let mut args = args.into_iter();
            $(
                let $name: $t = match args.next() {
                    Some($name) => match $name.try_into() {
                        Ok($name) => $name,
                        Err(_) => Err("Can not parse value")?
                    }
                    None => Err("Missing arguments")?
                };
            )+

            let inner = |$($name: $t),+| $b;

            let result = inner($($name),+);
            Ok(result.into())
        }
    };
}

pub type Filter = dyn Fn(Vec<&Value>) -> Result<Value, &str>;

pub const STR_LEN: &Filter = &create_filter!(|input: String| -> isize {
    input.len()
});