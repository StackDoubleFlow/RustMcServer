pub enum ChatColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    NoColor,
}

pub enum ClickEventType {
    OpenUrl,
    RunCommand,
    SuggestCommand,
    ChangePage,
}

pub enum HoverEventType {
    ShowText,
    ShowItem,
    ShowEntity,
    ChangePage,
}

pub struct ChatComponent {
    text: String,
    color: ChatColor,
    obfuscated: bool,
    bold: bool,
    strikethrough: bool,
    underline: bool,
    italic: bool,
    reset: bool,
    extra: Vec<ChatComponent>,
}

impl ChatComponent {
    pub fn new(text: String) -> Self {
        ChatComponent {
            text,
            color: ChatColor::NoColor,
            obfuscated: false,
            bold: false,
            strikethrough: false,
            underline: false,
            italic: false,
            reset: false,
            extra: Vec::new(),
        }
    }

    pub fn color_to_str(cc: ChatColor) -> &'static str {
        match cc {
            ChatColor::Black => "black",
            ChatColor::DarkBlue => "dark_blue",
            ChatColor::DarkGreen => "dark_green",
            ChatColor::DarkAqua => "dark_aqua",
            ChatColor::DarkRed => "dark_red",
            ChatColor::DarkPurple => "dark_purple",
            ChatColor::Gold => "gold",
            ChatColor::Gray => "gray",
            ChatColor::DarkGray => "dark_gray",
            ChatColor::Blue => "blue",
            ChatColor::Green => "green",
            ChatColor::Aqua => "aqua",
            ChatColor::Red => "red",
            ChatColor::LightPurple => "light_purple",
            ChatColor::Yellow => "yellow",
            ChatColor::White => "white",
            ChatColor::NoColor => "",
        }
    }
}
