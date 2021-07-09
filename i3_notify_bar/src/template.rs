use std::collections::HashMap;

use crate::notification_bar::NotificationTemplateData;

use mini_template::{value::Value, MiniTemplate};

static mut TEMPLATE_MANAGER: Option<MiniTemplate<u64>> = None;
static mut NEXT_TEMPLATE_ID: u64 = 0;

pub fn render_template(tpl_id: &u64, context: &NotificationTemplateData) -> String {
    let mut data = HashMap::with_capacity(5);

    data.insert(
        "app_name".to_string(),
        Value::String(context.app_name.to_owned()),
    );
    data.insert("body".to_string(), Value::String(context.body.to_owned()));
    data.insert(
        "expire_timeout".to_string(),
        Value::Number(context.expire_timeout as f64),
    );
    data.insert("icon".to_string(), Value::String(context.icon.to_owned()));
    data.insert(
        "summary".to_string(),
        Value::String(context.summary.to_owned()),
    );

    unsafe {
        let tplm = match &TEMPLATE_MANAGER {
            Some(tm) => tm,
            None => {
                TEMPLATE_MANAGER = Some(init_template_manager());
                TEMPLATE_MANAGER.as_ref().unwrap()
            }
        };
        let output = match tplm.render(tpl_id, &data) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        };
        output.replace('\n', "")
    }
}

pub fn add_template(template: String) -> Result<u64, ()> {
    unsafe {
        let old_template = match &mut TEMPLATE_MANAGER {
            Some(tm) => tm,
            None => {
                TEMPLATE_MANAGER = Some(init_template_manager());
                TEMPLATE_MANAGER.as_mut().unwrap()
            }
        }
        .add_template(NEXT_TEMPLATE_ID, template);

        match old_template {
            Ok(Some(_)) => {
                NEXT_TEMPLATE_ID += 1;
                Ok(NEXT_TEMPLATE_ID - 1)
            }
            Ok(None) => Ok(NEXT_TEMPLATE_ID),
            Err(_) => Err(()), // TODO return better error
        }
    }
}

fn init_template_manager() -> MiniTemplate<u64> {
    let mut tplm = MiniTemplate::default();
    tplm.add_default_modifiers();
    tplm
}
