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

icons!(
    "Discord" => DISCORD_ICON = '\u{F392}',
    "Slack" => SLACK_ICON = '\u{F198}',
    "Spotify" => SPOTIFY_ICON = '\u{F1BC}',
    "Steam" => STEAM_ICON = '\u{F1B6}',
    "Thunderbird" => MAIL_ICON = '\u{F2B6}',
    "x" => X_ICON = '\u{F057}'
);