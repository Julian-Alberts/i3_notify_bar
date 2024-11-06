use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq)]
pub struct AnimatedString<S: AsRef<str>> {
    pub start_offset: f64,
    pub max_width: usize,
    pub move_chars_per_sec: usize,
    pub text: S,
    pub stop_animation_for_secs: f64,
    text_reached_end: bool,
}

impl<S> AnimatedString<S>
where
    S: AsRef<str>,
{
    pub fn new(text: S) -> Self {
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

    pub fn set_text(&mut self, text: S) {
        self.text = text;
        self.start_offset = 0.;
        self.stop_animation_for_secs = 0.;
    }

    pub fn with_text(mut self, text: S) -> Self {
        self.text = text;
        self.start_offset = 0.;
        self.stop_animation_for_secs = 0.;
        self
    }
}

impl<S> ComponentString for AnimatedString<S>
where
    S: AsRef<str>,
{
    fn to_component_text(&self) -> String {
        let text_len = self.text.as_ref().chars().count();

        if text_len <= self.max_width {
            return self.text.as_ref().to_owned();
        }
        let end = if self.start_offset as usize + self.max_width < text_len {
            self.start_offset as usize + self.max_width
        } else {
            text_len
        };

        let chars = self.text.as_ref().chars().collect::<Vec<char>>();
        let chars = &chars[self.start_offset as usize..end];

        format!(
            "{text: <width$}",
            text = chars.iter().collect::<String>(),
            width = self.max_width
        )
    }

    fn update(&mut self, dt: f64) {
        let text_len = self.text.as_ref().chars().count();

        if text_len <= self.max_width {
            return;
        }

        // As long as stop_animation_for_secs is greater than 0 the animation is stopped
        if self.stop_animation_for_secs > 0.0 {
            self.stop_animation_for_secs -= dt;
            if self.stop_animation_for_secs > 0. {
                return;
            }
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

impl<S> From<AnimatedString<S>> for Box<dyn ComponentString>
where
    S: AsRef<str> + 'static,
{
    fn from(s: AnimatedString<S>) -> Self {
        Box::new(s)
    }
}

impl<S> Display for AnimatedString<S>
where
    S: AsRef<str>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_component_text(), f)
    }
}

pub struct PartiallyAnimatedString<Sl, Sc, Sr>
where
    Sl: AsRef<str>,
    Sc: AsRef<str>,
    Sr: AsRef<str>,
{
    left_static: Option<Sl>,
    animated_text: AnimatedString<Sc>,
    right_static: Option<Sr>,
}

impl<Sl, Sc, Sr> PartiallyAnimatedString<Sl, Sc, Sr>
where
    Sl: AsRef<str>,
    Sc: AsRef<str>,
    Sr: AsRef<str>,
{
    pub fn new(left: Option<Sl>, animated: Sc, right: Option<Sr>) -> Self {
        Self {
            animated_text: AnimatedString::new(animated),
            left_static: left,
            right_static: right,
        }
    }

    pub fn set_left_static(&mut self, left: Option<Sl>) {
        self.left_static = left;
    }

    pub fn with_left_static<Sln: AsRef<str>>(
        self,
        left: Option<Sln>,
    ) -> PartiallyAnimatedString<Sln, Sc, Sr> {
        PartiallyAnimatedString {
            left_static: left,
            animated_text: self.animated_text,
            right_static: self.right_static,
        }
    }

    pub fn set_animated_text(&mut self, animated: Sc) {
        self.animated_text.set_text(animated);
    }

    pub fn with_animated_text<Scn: AsRef<str>>(
        self,
        animated: Scn,
    ) -> PartiallyAnimatedString<Sl, Scn, Sr> {
        PartiallyAnimatedString {
            left_static: self.left_static,
            animated_text: AnimatedString::new(animated),
            right_static: self.right_static,
        }
    }

    pub fn set_right_static(&mut self, right: Option<Sr>) {
        self.right_static = right;
    }

    pub fn with_right_static<Srn: AsRef<str>>(
        self,
        right: Option<Srn>,
    ) -> PartiallyAnimatedString<Sl, Sc, Srn> {
        PartiallyAnimatedString {
            left_static: self.left_static,
            animated_text: self.animated_text,
            right_static: right,
        }
    }

    pub fn set_max_width(&mut self, max_width: usize) {
        self.animated_text.set_max_width(max_width);
    }

    pub fn with_max_width(mut self, max_width: usize) -> Self {
        self.animated_text.set_max_width(max_width);
        self
    }

    pub fn set_move_chars_per_sec(&mut self, move_chars_per_sec: usize) {
        self.animated_text
            .set_move_chars_per_sec(move_chars_per_sec);
    }

    pub fn with_move_chars_per_sec(mut self, move_chars_per_sec: usize) -> Self {
        self.animated_text
            .set_move_chars_per_sec(move_chars_per_sec);
        self
    }
}

impl<Sl, Sc, Sr> ComponentString for PartiallyAnimatedString<Sl, Sc, Sr>
where
    Sl: AsRef<str>,
    Sc: AsRef<str>,
    Sr: AsRef<str>,
{
    fn to_component_text(&self) -> String {
        let mut out_text = String::new();
        if let Some(s) = &self.left_static {
            out_text.push_str(s.as_ref());
        }
        out_text.push_str(self.animated_text.to_component_text().as_str());
        if let Some(s) = &self.right_static {
            out_text.push_str(s.as_ref());
        }

        out_text
    }

    fn update(&mut self, dt: f64) {
        self.animated_text.update(dt)
    }
}

impl<Sl, Sc, Sr> Display for PartiallyAnimatedString<Sl, Sc, Sr>
where
    Sl: AsRef<str>,
    Sc: AsRef<str>,
    Sr: AsRef<str>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.to_component_text(), f)
    }
}

impl ComponentString for String {
    fn to_component_text(&self) -> String {
        self.clone()
    }

    fn update(&mut self, _: f64) {}
}

pub trait ComponentString: Display {
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

#[cfg(test)]
mod tests {
    use crate::string::ComponentString;

    use super::{AnimatedString, PartiallyAnimatedString};

    #[test]
    fn new_animated_string() {
        let string = "Test animated string";
        let ani_str = AnimatedString::new(string);
        assert_eq!(
            AnimatedString {
                start_offset: 0.0,
                max_width: 20,
                move_chars_per_sec: 5,
                text: string,
                stop_animation_for_secs: 0.0,
                text_reached_end: false,
            },
            ani_str
        );
    }

    #[test]
    fn set_max_width() {
        let mut s = AnimatedString::new("");
        assert_eq!(s.max_width, 20);
        s.set_max_width(10);
        assert_eq!(s.max_width, 10);
        s = s.with_max_width(15);
        assert_eq!(s.max_width, 15);
    }

    #[test]
    fn set_move_chars_per_sec() {
        let mut s = AnimatedString::new("");
        assert_eq!(s.move_chars_per_sec, 5);
        s.set_move_chars_per_sec(10);
        assert_eq!(s.move_chars_per_sec, 10);
        s = s.with_move_chars_per_sec(15);
        assert_eq!(s.move_chars_per_sec, 15);
    }

    #[test]
    fn set_text() {
        let mut s = AnimatedString::new("");
        assert_eq!(s.text, "");
        s.set_text("a");
        assert_eq!(s.text, "a");
        s = s.with_text("b");
        assert_eq!(s.text, "b");
    }

    #[test]
    fn to_component_text_short() {
        let s = AnimatedString::new("foo");
        assert_eq!(s.to_component_text().as_str(), "foo");
    }

    #[test]
    fn to_component_text_long() {
        let s = AnimatedString::new("foobarfoobarfoobarfoxbar");
        assert_eq!(s.to_component_text().as_str(), "foobarfoobarfoobarfo");
    }

    #[test]
    fn to_component_text_with_updates() {
        let mut s = AnimatedString::new("abcdefghijklmnopqrstuvwxyz")
            .with_max_width(3)
            .with_move_chars_per_sec(1);
        assert_eq!(s.to_component_text().as_str(), "abc");

        s.update(1.);
        assert_eq!(s.to_component_text().as_str(), "bcd");

        s.update(10.);
        assert_eq!(s.to_component_text().as_str(), "lmn");

        s.update(11.);
        assert_eq!(s.to_component_text().as_str(), "wxy");
        assert!(!s.text_reached_end);
        assert_eq!(s.stop_animation_for_secs, 0.);

        s.update(1.);
        assert_eq!(s.to_component_text().as_str(), "xyz");
        assert!(s.text_reached_end);
        assert_eq!(s.stop_animation_for_secs, 1.);

        s.update(0.5);
        assert_eq!(s.to_component_text().as_str(), "xyz");
        assert!(s.text_reached_end);
        assert_eq!(s.stop_animation_for_secs, 0.5);

        s.update(0.5);
        assert_eq!(s.to_component_text().as_str(), "abc");
        assert!(!s.text_reached_end);
        assert_eq!(s.stop_animation_for_secs, 1.);
    }

    #[test]
    fn partially_animated_center_only() {
        let mut s =
            PartiallyAnimatedString::<&str, _, &str>::new(None, "abcdefghijklmnopqrstuvwxyz", None)
                .with_max_width(3)
                .with_move_chars_per_sec(1);
        assert_eq!(s.to_component_text().as_str(), "abc");
        s.update(1.);
        assert_eq!(s.to_component_text().as_str(), "bcd");
    }

    #[test]
    fn partially_animated() {
        let mut s = PartiallyAnimatedString::<_, _, &str>::new(
            Some("left"),
            "abcdefghijklmnopqrstuvwxyz",
            Some("right"),
        )
        .with_max_width(3)
        .with_move_chars_per_sec(1);
        assert_eq!(s.to_component_text().as_str(), "leftabcright");
        s.update(1.);
        assert_eq!(s.to_component_text().as_str(), "leftbcdright");
    }
}
