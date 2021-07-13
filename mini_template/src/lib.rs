mod compiler;
mod error;
mod modifier;
mod parser;
mod prelude;
mod renderer;
pub mod value;

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate log;

use modifier::Modifier;
use parser::{parse, ParseError};
use renderer::render;
use std::{collections::HashMap, fmt::Display, hash::Hash};
use value::Value;

#[derive(Default)]
pub struct MiniTemplate<K: Eq + Hash + Display> {
    modifier: HashMap<String, &'static Modifier>,
    template: HashMap<K, Template>,
}

impl<K: Eq + Hash + Display> MiniTemplate<K> {
    #[deprecated]
    pub fn new() -> Self {
        MiniTemplate {
            modifier: HashMap::new(),
            template: HashMap::new(),
        }
    }

    pub fn add_default_modifiers(&mut self) {
        use modifier::*;
        self.add_modifier("slice".to_owned(), &slice_modifier);
        self.add_modifier("regex".to_owned(), &match_modifier);
        self.add_modifier("replace".to_owned(), &replace_modifier);
        self.add_modifier("replace_regex".to_owned(), &replace_regex_modifier);
        self.add_modifier("upper".to_owned(), &upper);
        self.add_modifier("lower".to_owned(), &lower);

        self.add_modifier("add".to_owned(), &add);
        self.add_modifier("sub".to_owned(), &sub);
        self.add_modifier("mul".to_owned(), &mul);
        self.add_modifier("div".to_owned(), &div);
    }

    pub fn add_modifier(&mut self, key: String, modifier: &'static Modifier) {
        self.modifier.insert(key, modifier);
    }

    pub fn add_template(&mut self, key: K, tpl: String) -> Result<Option<Template>, ParseError> {
        let tpl = parse(tpl)?;
        Ok(self.template.insert(key, tpl))
    }

    pub fn render(&self, key: &K, data: &HashMap<String, Value>) -> error::Result<String> {
        let tpl = match self.template.get(key) {
            Some(t) => t,
            None => return Err(error::ErrorKind::UnknownTemplate),
        };
        render(tpl, &self.modifier, data)
    }
}

#[derive(Debug, PartialEq)]
pub struct Template {
    tpl_str: String,
    tpl: Vec<Statement>,
}

#[derive(Debug)]
enum Statement {
    Literal(*const str),
    Calculated {
        var_name: *const str,
        modifiers: Vec<(*const str, Vec<StorageMethod>)>,
    },
}

#[derive(Debug, PartialEq)]
enum StorageMethod {
    Const(Value),
    Variable(*const str),
}
