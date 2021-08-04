use std::{collections::HashMap, iter::Peekable, str::FromStr};

lazy_static! {
    static ref EMOJI_TREE: EmojiTree = EmojiTree::from_str(include_str!("../../emojis")).unwrap();
}

pub fn handle(text: String) -> String {
    let mut chars = text.chars().peekable();
    let mut has_more_chars = chars.peek().is_some();

    let mut new_text = String::with_capacity(text.capacity());
    while has_more_chars {
        if let Some(emoji_name) = EMOJI_TREE.find_emoji(&mut chars) {
            new_text.push_str(emoji_name);
        } else {
            new_text.push(chars.next().unwrap());
        }
        has_more_chars = chars.peek().is_some();
    }

    new_text
}

#[derive(Debug, Default)]
struct EmojiTree {
    branches: HashMap<u32, EmojitreeEntry>,
}

impl EmojiTree {
    pub fn insert(&mut self, mut path: impl Iterator<Item = u32>, value: String) -> Option<String> {
        let key = path.next().unwrap();

        match self.branches.get_mut(&key) {
            Some(entry) => entry,
            None => {
                let entry = EmojitreeEntry::default();
                self.branches.insert(key, entry);
                self.branches.get_mut(&key).unwrap()
            }
        }
        .insert(path, value)
    }

    pub fn find_emoji<'slice, 'name>(
        &'name self,
        chars: &mut Peekable<impl Iterator<Item = char>>,
    ) -> Option<&'name str> {
        let key = *chars.peek()? as u32;

        let entry = self.branches.get(&key)?;
        chars.next();

        let emoji_name = entry.find_emoji(chars)?;

        Some(emoji_name)
    }
}

impl FromStr for EmojiTree {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tree = EmojiTree {
            branches: HashMap::default(),
        };
        for (line_count, line) in s.lines().enumerate() {
            // TODO replace with line.split_once(' ') after it has become stable.
            let split = {
                let mut split = line.splitn(2, ' ');
                let hex = split.next();
                let name = split.next();

                match (hex, name) {
                    (Some(h), Some(n)) => Some((h, n)),
                    _ => None,
                }
            };

            let (hex, name) = match split {
                Some(split) => split,
                None => return Err(format!("Could not find whitespace in line {}", line_count)),
            };

            let chars_iter = hex.split('_').map(|hex| {
                u32::from_str_radix(hex, 16)
                    .unwrap_or_else(|_| panic!("Can not parse {} as u32", hex))
            });

            let mut name = name
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .fold(String::from(':'), |mut s, c| {
                    s.push(c);
                    s
                });
            name.push(':');
            name = name.replace("__", "_");
            tree.insert(chars_iter, name);
        }

        Ok(tree)
    }
}

#[derive(Debug, Default)]
pub struct EmojitreeEntry {
    leaf: Option<String>,
    branch: Option<EmojiTree>,
}

impl EmojitreeEntry {
    pub fn insert(&mut self, mut path: impl Iterator<Item = u32>, value: String) -> Option<String> {
        let key = match path.next() {
            Some(key) => key,
            None => return self.leaf.replace(value),
        };

        let branch = match &mut self.branch {
            Some(b) => b,
            None => {
                let branch = EmojiTree::default();
                self.branch = Some(branch);
                self.branch.as_mut().unwrap()
            }
        };

        match branch.branches.get_mut(&key) {
            Some(entry) => entry,
            None => {
                let entry = EmojitreeEntry::default();
                branch.branches.insert(key, entry);
                branch.branches.get_mut(&key).unwrap()
            }
        }
        .insert(path, value)
    }

    pub fn find_emoji<'name>(
        &'name self,
        path: &mut Peekable<impl Iterator<Item = char>>,
    ) -> Option<&'name str> {
        let key = match path.peek() {
            Some(k) => *k as u32,
            None => return self.leaf.as_ref().map(|s| &s[..]),
        };

        let branch = match &self.branch {
            Some(b) => b,
            None => return self.leaf.as_ref().map(|s| &s[..]),
        };

        let entry = match branch.branches.get(&key) {
            Some(entry) => entry,
            None => return self.leaf.as_ref().map(|s| &s[..]),
        };
        path.next();
        entry.find_emoji(path)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn text_without_emojis() {
        let text = String::from("Hello world!");
        assert_eq!(super::handle(text), String::from("Hello world!"));
    }

    #[test]
    fn text_with_umlaut() {
        let text = String::from("Geht nach Brüssel");
        assert_eq!(&super::handle(text)[..], "Geht nach Brüssel");
    }

    #[test]
    fn text_with_single_char_emoji() {
        let text = String::from("Hello \u{1F609} world!");
        assert_eq!(
            super::handle(text),
            String::from("Hello :winking_face: world!")
        );
    }

    #[test]
    fn text_with_multi_char_emoji() {
        let text = String::from("Hello \u{1F468}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468} world!");
        assert_eq!(
            super::handle(text),
            String::from("Hello :couple_with_heart_man_man: world!")
        );
    }

    #[test]
    fn create_emoji_tree_from_string() {
        use std::str::FromStr;
        let tree = super::EmojiTree::from_str(r#"ff00_fffff_1234 test value"#);
        assert!(tree.is_ok(), "{:#?}", tree);
        let tree = tree.unwrap();
        let tree_entry = tree.branches.get(&0xff00);
        assert!(tree_entry.is_some());

        let tree_entry = tree_entry.unwrap();
        let branch = tree_entry.branch.as_ref();
        assert!(branch.is_some());
        let branch = branch.unwrap();
        let tree_entry = branch.branches.get(&0xfffff);
        assert!(tree_entry.is_some());

        let tree_entry = tree_entry.unwrap();
        let branch = tree_entry.branch.as_ref();
        assert!(branch.is_some());
        let branch = branch.unwrap();
        let tree_entry = branch.branches.get(&0x1234);
        assert!(tree_entry.is_some());

        let tree_entry = tree_entry.unwrap();
        assert_eq!(tree_entry.leaf, Some(String::from(":test_value:")))
    }
}
