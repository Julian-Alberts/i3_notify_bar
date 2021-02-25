use std::io::prelude::*;

pub fn compile(tpl: String) -> Template {
    let mut compiled_tpl = Template {
        tpl_str: tpl,
        tpl: Vec::new()
    };

    let mut tpl = &compiled_tpl.tpl_str[..];
    
    while !tpl.is_empty() {
        let segment_length;
        if tpl.starts_with('{') {
            if tpl.starts_with("{{}") || tpl.starts_with("{}}") {
                compiled_tpl.tpl.push(Segment::String(&tpl[1..2]));
                segment_length = 3;
            } else {
                unimplemented!()
            }
        } else {
            segment_length = match tpl.find('{') {
                Some(s) => s,
                None => tpl.len()
            };

            compiled_tpl.tpl.push(Segment::String(&tpl[..segment_length]));
        }
        tpl = &tpl[segment_length..];
    }

    compiled_tpl
}

#[derive(Debug)]
enum Segment {
    String(*const str)
}

impl PartialEq for Segment {

    fn eq(&self, other: &Segment) -> bool {
        match (self, other) {
            (Segment::String(s), Segment::String(o)) => unsafe {
                s.as_ref() == o.as_ref()
            }
        }
    }

}

pub struct Template {
    tpl_str: String,
    tpl: Vec<Segment>
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn simple_compile() {
        let tpl = compile("Simple template string".to_owned());
        assert_eq!(vec![Segment::String("Simple template string" as *const _)], tpl.tpl);
    }

    #[test]
    fn bracket_literal() {
        let tpl = compile("Simple {{} template {}} string".to_owned());
        assert_eq!(vec![
            Segment::String("Simple " as *const _),
            Segment::String("{" as *const _),
            Segment::String(" template " as *const _),
            Segment::String("}" as *const _),
            Segment::String(" string" as *const _),
        ], tpl.tpl);
    }

}