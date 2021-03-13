mod compiler;
mod modifier;
mod prelude;
pub mod value;
mod renderer;
mod error;

use std::{collections::HashMap, fmt::Display, hash::Hash};
use compiler::compile;
use modifier::{Modifier, match_modifier, slice_modifier};
use renderer::render;
use value::Value;


pub struct MiniTemplate<K: Eq + Hash + Display> {
    modifier: HashMap<String, &'static Modifier>,
    template: HashMap<K, Template>
}

impl <K: Eq + Hash + Display> MiniTemplate<K> {

    pub fn new() -> Self {
        MiniTemplate {
            modifier: HashMap::new(),
            template: HashMap::new()
        }
    }

    pub fn add_default_modifiers(&mut self) {
        self.add_modifier("slice".to_owned(), &slice_modifier);
        self.add_modifier("regex".to_owned(), &match_modifier);
    }

    pub fn add_modifier(&mut self, key: String, modifier: &'static Modifier) {
        self.modifier.insert(key, modifier);
    }

    pub fn add_template(&mut self, key: K, tpl: String) {
        self.template.insert(key, compile(tpl));
    }

    pub fn render(&self, key: &K, data: &HashMap<String, Value>) -> error::Result<String> {
        let tpl = match self.template.get(key) {
            Some(t) => t,
            None => return Err(error::ErrorKind::UnknownTemplate)
        };
        render(tpl, &self.modifier, data)
    }

}

pub struct Template {
    tpl_str: String,
    tpl: Vec<Statement>
}

#[derive(Debug)]
enum Statement {
    Literal(*const str),
    Calculated {
        var_name: *const str,
        modifiers: Vec<(*const str, Vec<StorageMethod>)>
    },
}

#[derive(Debug)]
enum StorageMethod {
    Const(Value),
    Variable(*const str)
}