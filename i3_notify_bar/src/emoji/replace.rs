use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{prelude::*, BufReader},
    iter::Peekable,
    path::Path,
    str::FromStr,
    sync::Mutex,
};

use lazy_static::lazy_static;
use log::error;

lazy_static! {
    static ref EMOJI_TREE: Mutex<EmojiTree> = Mutex::default();
}

pub fn load_emoji_file(path: &Path) {
    let file = match OpenOptions::new().read(true).open(path) {
        Ok(f) => f,
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    log::error!("File not found \"{}\"", path.to_string_lossy())
                }
                std::io::ErrorKind::PermissionDenied => {
                    log::error!("Permission denied \"{}\"", path.to_string_lossy())
                }
                _ => log::error!("Unexpected error \"{}\"", path.to_string_lossy()),
            }
            return;
        }
    };

    let mut text = String::new();
    match BufReader::new(file).read_to_string(&mut text) {
        Ok(_) => {}
        Err(_) => {
            log::error!("Invalid UTF-8 \"{}\"", path.to_string_lossy());
            return;
        }
    };

    let mut et = match EMOJI_TREE.lock() {
        Ok(et) => et,
        Err(e) => {
            error!("Could not lock emoji tree {}", e);
            return;
        }
    };
    match EmojiTree::from_str(&text) {
        Ok(emoji_tree) => *et = emoji_tree,
        Err(e) => {
            error!("Could not parse emoji file {}", e)
        }
    };
}

pub fn handle(text: String) -> String {
    let mut chars = text.chars().peekable();
    let mut has_more_chars = chars.peek().is_some();

    let mut new_text = String::with_capacity(text.capacity());
    let emoji_tree = match EMOJI_TREE.lock() {
        Ok(et) => et,
        Err(_) => return text,
    };
    while has_more_chars {
        if let Some(emoji_name) = emoji_tree.find_emoji(&mut chars) {
            new_text.push_str(emoji_name);
        } else {
            // has_more_chars makes sure that chars has a next item
            new_text.push(chars.next().unwrap());
        }
        has_more_chars = chars.peek().is_some();
    }

    new_text
}

/// Store and find emoji based on chars
#[derive(Debug, Default)]
struct EmojiTree {
    branches: HashMap<u32, EmojitreeEntry>,
}

impl EmojiTree {
    /// Insert new emoji replacement.
    /// Path should be an iterator of u32 which each represend a single char in a given emoji.
    /// Returns old replacement if a replacement has allready been defined at a given path.
    pub fn insert(&mut self, mut path: impl Iterator<Item = u32>, value: String) -> Option<String> {
        let key = path.next()?;

        match self.branches.get_mut(&key) {
            Some(entry) => entry,
            None => {
                let entry = EmojitreeEntry::default();
                self.branches.insert(key, entry);
                self.branches.get_mut(&key)?
            }
        }
        .insert(path, value)
    }

    /// Iterates over iterator until the end of an emoji has been found.
    /// Returns Option::None if char iterator does not start with a emoji or iterator is empty
    pub fn find_emoji<'name>(
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
    /// Generate emoji tree from String
    /// Returns with error if a single line coud not be read.
    ///
    /// Each line will be interpreted as a entry. Empty lines will rais errors.
    /// each line start with the utf8 code of an emoji as a hexadecimal number.
    /// If an emoji consists of more than one char they sould be sperated by a '_'
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

            tree.insert(chars_iter, name.to_owned());
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
                self.branch.as_mut()?
            }
        };

        match branch.branches.get_mut(&key) {
            Some(entry) => entry,
            None => {
                let entry = EmojitreeEntry::default();
                branch.branches.insert(key, entry);
                branch.branches.get_mut(&key)?
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
    use std::path::Path;

    #[test]
    fn text_without_emojis() {
        super::load_emoji_file(Path::new("emojis"));
        let text = String::from("Hello world!");
        assert_eq!(super::handle(text), String::from("Hello world!"));
    }

    #[test]
    fn text_with_umlaut() {
        super::load_emoji_file(Path::new("emojis"));
        let text = String::from("Geht nach Brüssel");
        assert_eq!(&super::handle(text)[..], "Geht nach Brüssel");
    }

    #[test]
    fn text_with_single_char_emoji() {
        super::load_emoji_file(Path::new("emojis"));
        let text = String::from("Hello \u{1F609} world!");
        assert_eq!(
            super::handle(text),
            String::from("Hello :winking_face: world!")
        );
    }

    #[test]
    fn text_with_multi_char_emoji() {
        super::load_emoji_file(Path::new("emojis"));
        let text = String::from("Hello \u{1F468}\u{200D}\u{2764}\u{FE0F}\u{200D}\u{1F468} world!");
        assert_eq!(
            super::handle(text),
            String::from("Hello :couple_with_heart_man_man: world!")
        );
    }

    #[test]
    fn create_emoji_tree_from_string() {
        use std::str::FromStr;
        super::load_emoji_file(Path::new("emojis"));
        let tree = super::EmojiTree::from_str(r#"ff00_fffff_1234 :test_value:"#);
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
