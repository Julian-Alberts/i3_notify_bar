use std::ops::RangeInclusive;

// TODO add real ranges
const EMOJI_RANGES: [RangeInclusive<char>; 26] = [
    '\u{0080}'..='\u{02AF}',
    '\u{0300}'..='\u{03FF}',
    '\u{0600}'..='\u{06FF}',
    '\u{0C00}'..='\u{0C7F}',
    '\u{1DC0}'..='\u{1DFF}',
    '\u{1E00}'..='\u{1EFF}',
    '\u{2000}'..='\u{209F}',
    '\u{20D0}'..='\u{214F}',
    '\u{2190}'..='\u{23FF}',
    '\u{2460}'..='\u{25FF}',
    '\u{2600}'..='\u{27EF}',
    '\u{2900}'..='\u{29FF}',
    '\u{2B00}'..='\u{2BFF}',
    '\u{2C60}'..='\u{2C7F}',
    '\u{2E00}'..='\u{2E7F}',
    '\u{3000}'..='\u{303F}',
    '\u{A490}'..='\u{A4CF}',
    '\u{E000}'..='\u{F8FF}',
    '\u{FE00}'..='\u{FE0F}',
    '\u{FE30}'..='\u{FE4F}',
    '\u{1F000}'..='\u{1F02F}',
    '\u{1F0A0}'..='\u{1F0FF}',
    '\u{1F100}'..='\u{1F64F}',
    '\u{1F680}'..='\u{1F6FF}',
    '\u{1F910}'..='\u{1F96B}',
    '\u{1F980}'..='\u{1F9E0}',
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
