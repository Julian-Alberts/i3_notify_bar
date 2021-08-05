use std::ops::RangeInclusive;

// TODO add real ranges
const EMOJI_RANGES: [RangeInclusive<char>; 3] = [
    '\u{200D}'..='\u{3F00}',
    '\u{FE0F}'..='\u{1FAD6}',
    '\u{E0062}'..='\u{E007F}',
];

pub fn handle(text: String) -> String {
    text.chars()
        .filter(|c| !EMOJI_RANGES.iter().any(|range| range.contains(c)))
        .collect::<_>()
}

#[cfg(test)]
mod tests {

    #[test]
    fn text_without_emojis() {
        let text = String::from("Hello world!");
        assert_eq!(super::handle(text), String::from("Hello world!"));
    }

    #[test]
    fn text_with_single_char_emoji() {
        let text = String::from("Hello \u{1F609} world!");
        assert_eq!(super::handle(text), String::from("Hello  world!"));
    }

    #[test]
    fn text_with_multi_char_emoji() {
        let text = String::from("Hello \u{1F468}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468} world!");
        assert_eq!(super::handle(text), String::from("Hello  world!"));
    }
}
