macro_rules! icons {
    ($($name: tt => $icon_name: ident $(= $char: literal)?),*) => {
        $($(
            pub const $icon_name: char = $char;
        )?)*

        pub fn get_icon(name: &str) -> Option<char> {
            let c = match name {
                $(
                    $name => $icon_name,
                )*
                _ => return None
            };

            Some(c)
        }


    };
}

#[cfg(feature = "fa_icons")]
icons!(
    "Discord" => DISCORD_ICON = '\u{F392}',
    "Download" => X_DOWNLOAD = '\u{F019}',
    "FileZilla" => X_DOWNLOAD,
    "Mail" => MAIL_ICON = '\u{F2B6}',
    "Slack" => SLACK_ICON = '\u{F198}',
    "Spotify" => SPOTIFY_ICON = '\u{F1BC}',
    "Steam" => STEAM_ICON = '\u{F1B6}',
    "Thunderbird" => MAIL_ICON,
    "x" => X_ICON = '\u{F057}'
);

#[cfg(not(feature = "fa_icons"))]
icons!(
    "x" => X_ICON = 'X'
);

#[cfg(test)]
mod tests {

    #[test]
    fn macro_test() {
        icons!(
            "Test" => TEST_ICON = 'w'
        );

        assert_eq!(get_icon("Test"), Some('w'));
        assert_eq!(get_icon(""), None);
        assert_eq!(TEST_ICON, 'w');
    }
}
