mod compiler;
mod filter;
mod prelude;
mod value;

use std::collections::HashMap;
use filter::Filter;

pub struct MiniTemplate<'a> {
    filters: HashMap<String, &'a Filter>
}

impl <'a> MiniTemplate<'a> {

    pub fn new() -> Self {
        MiniTemplate {
            filters: HashMap::new()
        }
    }

    pub fn add_filter(&mut self, key: String, filter: &'a Filter) {
        self.filters.insert(key, filter);
    }

    pub fn add_template(&mut self, key: String, tpl: String) {

    }

}