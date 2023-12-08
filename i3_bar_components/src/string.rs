pub struct AnimatedString {
    pub start_offset: f64,
    pub max_width: usize,
    pub move_chars_per_sec: usize,
    pub text: String,
    pub stop_animation_for_secs: f64,
    text_reached_end: bool,
}

impl AnimatedString {
    pub fn new(text: String) -> Self {
        Self {
            start_offset: 0.,
            max_width: 20,
            move_chars_per_sec: 5,
            text,
            stop_animation_for_secs: 0.,
            text_reached_end: false,
        }
    }

    pub fn set_max_width(&mut self, max_width: usize) {
        self.max_width = max_width;
    }

    pub fn with_max_width(mut self, max_width: usize) -> Self {
        self.max_width = max_width;
        self
    }

    pub fn set_move_chars_per_sec(&mut self, move_chars_per_sec: usize) {
        self.move_chars_per_sec = move_chars_per_sec;
    }

    pub fn with_move_chars_per_sec(mut self, move_chars_per_sec: usize) -> Self {
        self.move_chars_per_sec = move_chars_per_sec;
        self
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.start_offset = 0.;
        self.stop_animation_for_secs = 0.;
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = text;
        self.start_offset = 0.;
        self.stop_animation_for_secs = 0.;
        self
    }
}

impl ComponentString for AnimatedString {
    fn to_component_text(&self) -> String {
        let text_len = self.text.chars().count();

        if text_len <= self.max_width {
            return self.text.to_owned();
        }
        let end = if self.start_offset as usize + self.max_width < text_len {
            self.start_offset as usize + self.max_width
        } else {
            text_len
        };

        let chars = self.text.chars().collect::<Vec<char>>();
        let chars = &chars[self.start_offset as usize..end];

        format!(
            "{text: <width$}",
            text = chars.iter().collect::<String>(),
            width = self.max_width
        )
    }

    fn update(&mut self, dt: f64) {
        let text_len = self.text.chars().count();

        if text_len <= self.max_width {
            return;
        }

        // As long as stop_animation_for_secs is greater than 0 the animation is stopped
        if self.stop_animation_for_secs > 0.0 {
            self.stop_animation_for_secs -= dt;
            return;
        }

        if self.text_reached_end {
            self.start_offset = 0.;
            self.stop_animation_for_secs = 1.;
            self.text_reached_end = false;
            return;
        }

        let move_chars = self.move_chars_per_sec as f64 * dt;
        self.start_offset += move_chars;
        if self.start_offset as usize >= text_len - self.max_width {
            self.text_reached_end = true;
            self.start_offset = (text_len - self.max_width) as f64;
            self.stop_animation_for_secs = 1.0;
        }
    }
}

impl From<AnimatedString> for Box<dyn ComponentString> {
    fn from(s: AnimatedString) -> Self {
        Box::new(s)
    }
}

pub struct PartiallyAnimatedString {
    left_static: Option<String>,
    animated_text: AnimatedString,
    right_static: Option<String>,
}

impl PartiallyAnimatedString {
    pub fn new(left: Option<String>, animated: AnimatedString, right: Option<String>) -> Self {
        Self {
            animated_text: animated,
            left_static: left,
            right_static: right,
        }
    }

    pub fn set_left_static(&mut self, left: Option<String>) {
        self.left_static = left;
    }

    pub fn with_left_static(mut self, left: Option<String>) -> Self {
        self.left_static = left;
        self
    }

    pub fn set_animated_text(&mut self, animated: AnimatedString) {
        self.animated_text = animated;
    }

    pub fn with_animated_text(mut self, animated: AnimatedString) -> Self {
        self.animated_text = animated;
        self
    }

    pub fn set_right_static(&mut self, right: Option<String>) {
        self.right_static = right;
    }

    pub fn with_right_static(mut self, right: Option<String>) -> Self {
        self.right_static = right;
        self
    }
}

impl ComponentString for PartiallyAnimatedString {
    fn to_component_text(&self) -> String {
        let mut out_text = String::new();
        if let Some(s) = &self.left_static {
            out_text.push_str(s.as_str());
        }
        out_text.push_str(self.animated_text.to_component_text().as_str());
        if let Some(s) = &self.right_static {
            out_text.push_str(s.as_str());
        }

        out_text
    }

    fn update(&mut self, dt: f64) {
        self.animated_text.update(dt)
    }
}

impl ComponentString for String {
    fn to_component_text(&self) -> String {
        self.clone()
    }

    fn update(&mut self, _: f64) {}
}

pub trait ComponentString {
    fn to_component_text(&self) -> String;
    fn update(&mut self, dt: f64);
}

impl ComponentString for Box<dyn ComponentString> {
    fn update(&mut self, dt: f64) {
        self.as_mut().update(dt);
    }
    fn to_component_text(&self) -> String {
        self.as_ref().to_component_text()
    }
}
