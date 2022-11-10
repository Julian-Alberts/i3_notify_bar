use crate::notification_bar::NotificationTemplateData;

use mini_template::{MiniTemplate, MiniTemplateBuilder};

pub const DEFAULT_TEMPLATE_ID: u64 = 0;

static mut TEMPLATE_MANAGER: Option<MiniTemplate> = None;
static mut NEXT_TEMPLATE_ID: u64 = DEFAULT_TEMPLATE_ID + 1;

pub fn render_template(tpl_id: &u64, context: &NotificationTemplateData) -> String {
    unsafe {
        let tplm = match &TEMPLATE_MANAGER {
            Some(tm) => tm,
            None => {
                TEMPLATE_MANAGER = Some(init_template_manager());
                match TEMPLATE_MANAGER.as_ref() {
                    Some(tm) => tm,
                    None => unreachable!(),
                }
            }
        };
        let output = match tplm.render(tpl_id.to_string().as_str(), context.clone().into()) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        };
        output.replace('\n', "")
    }
}

pub fn add_template(template: String) -> Result<u64, ()> {
    unsafe {
        let id = NEXT_TEMPLATE_ID.to_string();
        let old_template = match &mut TEMPLATE_MANAGER {
            Some(tm) => tm,
            None => {
                TEMPLATE_MANAGER = Some(init_template_manager());
                match TEMPLATE_MANAGER.as_mut() {
                    Some(tm) => tm,
                    None => unreachable!(),
                }
            }
        }
        .add_template(id, template);

        match old_template {
            Ok(_) => {
                NEXT_TEMPLATE_ID += 1;
                Ok(NEXT_TEMPLATE_ID - 1)
            }
            Err(_) => Err(()), // TODO return better error
        }
    }
}

fn init_template_manager() -> MiniTemplate {
    let mut tplm = MiniTemplateBuilder::default().with_default_modifiers().build();
    if let Err(_) = tplm.add_template("0".to_owned(), "[{{app_name}}] {{summary}}: {{body}}".to_owned()) {
        unreachable!("Invalid default template")
    }
    tplm
}
