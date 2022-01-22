use i3_bar_components::{components::{Label, prelude::{Component, Widget, SeperatorWidth, Seperator}, BaseComponent}, protocol::ClickEvent};

pub struct AnimatedLabel {
    pub start_offset: f64,
    pub max_width: usize,
    pub move_chars_per_sec: usize,
    pub text: String,
    pub stop_animation_for_secs: f64,
    pub label: Label,
    pub icon: char
}

impl AnimatedLabel {
    fn update_offset(&mut self, dt: f64) {
        let text_len = self.text.chars().count();

        if text_len <= self.max_width {
            return;
        }

        if self.stop_animation_for_secs > 0.0 {
            self.stop_animation_for_secs -= dt;
            return;
        }

        let move_chars = self.move_chars_per_sec as f64 * dt;
        self.start_offset += move_chars;
        if self.start_offset as usize >= text_len {
            self.start_offset = 0.0;
            self.stop_animation_for_secs = 1.0;
        }
    }
}

impl Component for AnimatedLabel {

    fn event(&mut self, event: &ClickEvent) {
        self.label.event(event)
    }
    fn collect_base_components<'a>(&'a self, base_components: &mut Vec<&'a BaseComponent>) {
        self.label.collect_base_components(base_components)
    }
    fn collect_base_components_mut<'a>(&'a mut self, base_components: &mut Vec<&'a mut BaseComponent>) {
        self.label.collect_base_components_mut(base_components)
    }
    fn get_id(&self) -> &str {
        self.label.get_id()
    }
    fn name(&self) -> &str {
        self.label.name()
    }
    fn update(&mut self, dt: f64) {
        self.update_offset(dt);
        self.label.set_text(self.to_string());
        self.label.update(dt);
    }

}

impl Widget for AnimatedLabel {

    fn get_base_component(&self) -> &BaseComponent {
        self.label.get_base_component()
    }

    fn get_base_component_mut(&mut self) -> &mut BaseComponent {
        self.label.get_base_component_mut()
    }

}

impl ToString for AnimatedLabel {
    fn to_string(&self) -> String {
        let text_len = self.text.chars().count();

        if text_len <= self.max_width {
            return self.text.to_owned();
        }
        let end;
        if self.start_offset as usize + self.max_width < text_len {
            end = self.start_offset as usize + self.max_width;
        } else {
            end = text_len;
        }

        let chars = self.text.chars().collect::<Vec<char>>();
        let chars = &chars[self.start_offset as usize..end];

        let main_text = format!(
            "{text: <width$}",
            text = chars.iter().collect::<String>(),
            width = self.max_width
        );

        if self.icon == ' ' {
            format!(" {} ", main_text)
        } else {
            format!(" {} {} ", self.icon, main_text)
        }
    }
}

impl SeperatorWidth for AnimatedLabel {}
impl Seperator for AnimatedLabel {}