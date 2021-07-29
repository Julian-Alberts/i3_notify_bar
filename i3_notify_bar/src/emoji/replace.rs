pub fn handle(text: String) -> String {
    text
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
        unimplemented!()
    }

    #[test]
    fn text_with_multi_char_emoji() {
        unimplemented!()
    }
}
