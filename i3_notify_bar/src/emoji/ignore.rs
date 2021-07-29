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
}
