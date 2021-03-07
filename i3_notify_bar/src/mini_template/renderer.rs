use std::collections::HashMap;

use super::{Statement, StorageMethod, Template, modifier::Modifier, value::Value};

pub fn render<'a>(tpl: &Template, modifier: &HashMap<String, &'a Modifier>, variables: &HashMap<String, Value>) -> Result<String, String> {
    let tpl = &tpl.tpl;
    let mut tpl_string = String::new();

    for statement in tpl {
        match statement {
            Statement::Literal(literal) => unsafe {
                // literal points to tpl.tpl_str and should never be null
                tpl_string.push_str(literal.as_ref().unwrap())
            },
            Statement::Calculated{var_name, modifiers} =>  {
                // var_name points to tpl.tpl_str and should never be null
                let var_name = unsafe {var_name.as_ref().unwrap()};
                let var = variables.get(var_name);
                let mut var = var.ok_or(format!("Unknown variable {}", var_name))?.to_owned();

                for (modifier_name, args) in modifiers {
                    // modifier_name points to tpl.tpl_str and should never be null
                    let modifier_name = unsafe {modifier_name.as_ref().unwrap()};
                    let modifier = modifier.get(modifier_name).ok_or(format!("Unknown modifier {}", modifier_name))?;

                    let args = storage_methods_to_values(args, variables)?;

                    var = modifier(&var, args)?;
                }

                tpl_string.push_str(&var.to_string()[..])
            }
        }
    }

    Ok(tpl_string)
}

fn storage_methods_to_values<'a>(args: &'a Vec<StorageMethod>, variables: &'a HashMap<String, Value>) -> Result<Vec<&'a Value>, String> {
    let mut real_args = Vec::with_capacity(args.len());
    
    for arg in args {
        let arg = match arg {
            StorageMethod::Const(value) => value,
            StorageMethod::Variable(var) => unsafe {
                // var points to tpl.tpl_str and should never be null
                let var = var.as_ref().unwrap();
                variables.get(var).ok_or(format!("Unknown variable \"{}\"", var))?
            }
        };
        real_args.push(arg);
    }
    Ok(real_args)
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{create_modifier, mini_template::{compiler::compile, modifier::Modifier, value::Value}};
    use std::convert::TryInto;

    use super::render;

    const UPPER_CASE_MODIFIER: &Modifier = &create_modifier!(|data: String| -> String {
        data.to_uppercase()
    });

    const ARGS_MODIFIER: &Modifier = &create_modifier!(|data: String, other: String, num: i32| -> String {
        format!("{}={}={}", data, other, num)
    });

    #[test]
    fn literal() {
        let tpl = String::from("Simple template string");
        let tpl = compile(tpl);
        let rendered = render(&tpl, &HashMap::new(), &HashMap::new()).unwrap();
        assert_eq!(rendered, tpl.tpl_str);
    }

    #[test]
    fn replace_variables() {
        let tpl = String::from("Simple {foo} template string");
        let tpl = compile(tpl);
        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));
        let rendered = render(&tpl, &HashMap::new(), &variables).unwrap();
        assert_eq!(rendered, String::from("Simple my test value template string"));
    }

    #[test]
    fn modifier() {
        let tpl = String::from("Simple {foo|upper} template string");
        let tpl = compile(tpl);
        
        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));

        let mut modifiers = HashMap::new();
        modifiers.insert("upper".to_owned(), UPPER_CASE_MODIFIER);

        let rendered = render(&tpl, &modifiers, &variables).unwrap();
        assert_eq!(rendered, String::from("Simple MY TEST VALUE template string"));
    }

    #[test]
    fn modifier_values() {
        let tpl = String::from(r#"Simple {foo|args:"BAR":42} template string"#);
        let tpl = compile(tpl);
        
        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));

        let mut modifiers = HashMap::new();
        modifiers.insert("args".to_owned(), ARGS_MODIFIER);

        let rendered = render(&tpl, &modifiers, &variables).unwrap();
        assert_eq!(rendered, String::from("Simple my test value=BAR=42 template string"));
    }

    #[test]
    fn modifier_list() {
        let tpl = String::from(r#"Simple {foo|upper|args:"bar":42} template string"#);
        let tpl = compile(tpl);
        
        let mut variables = HashMap::new();
        variables.insert("foo".to_owned(), Value::String("my test value".to_owned()));

        let mut modifiers = HashMap::new();
        modifiers.insert("args".to_owned(), ARGS_MODIFIER);
        modifiers.insert("upper".to_owned(), UPPER_CASE_MODIFIER);

        let rendered = render(&tpl, &modifiers, &variables).unwrap();
        assert_eq!(rendered, String::from("Simple MY TEST VALUE=bar=42 template string"));
    }


}