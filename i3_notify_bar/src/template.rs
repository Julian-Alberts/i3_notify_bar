use tinytemplate::TinyTemplate;

use crate::notification_bar::NotificationTemplateData;

static mut TEMPLATE_MANAGER: Option<TinyTemplate<'static>> = None;
static mut TEMPLATES: Vec<String> = Vec::new();

pub fn render_template(tpl_name: &str, context: &NotificationTemplateData) -> String {
    unsafe {
        match &TEMPLATE_MANAGER {
            Some(tm) => tm,
            None => {
                TEMPLATE_MANAGER = Some(TinyTemplate::new());
                TEMPLATE_MANAGER.as_ref().unwrap()
            }
        }.render(tpl_name, context).unwrap().replace('\n', "")
    }
}

pub fn add_template(template: String) -> Result<&'static str, ()> {

    unsafe {
        TEMPLATES.push(template);
        let temp_ref = TEMPLATES.last().unwrap();
            
        match &mut TEMPLATE_MANAGER {
            Some(tm) => tm,
            None => {
                TEMPLATE_MANAGER = Some(TinyTemplate::new());
                TEMPLATE_MANAGER.as_mut().unwrap()
            }
        }.add_template(temp_ref, temp_ref).or(Err(()))?;

        Ok(temp_ref)
    }
}