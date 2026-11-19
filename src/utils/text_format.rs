pub struct TextFormat;

impl TextFormat {
    pub const ESCAPE: char = '\u{00a7}';
    pub const BLACK: &'static str = "\u{00a7}0";
    pub const DARK_BLUE: &'static str = "\u{00a7}1";
    pub const DARK_GREEN: &'static str = "\u{00a7}2";
    pub const DARK_AQUA: &'static str = "\u{00a7}3";
    pub const DARK_RED: &'static str = "\u{00a7}4";
    pub const DARK_PURPLE: &'static str = "\u{00a7}5";
    pub const GOLD: &'static str = "\u{00a7}6";
    pub const GRAY: &'static str = "\u{00a7}7";
    pub const DARK_GRAY: &'static str = "\u{00a7}8";
    pub const BLUE: &'static str = "\u{00a7}9";
    pub const GREEN: &'static str = "\u{00a7}a";
    pub const AQUA: &'static str = "\u{00a7}b";
    pub const RED: &'static str = "\u{00a7}c";
    pub const LIGHT_PURPLE: &'static str = "\u{00a7}d";
    pub const YELLOW: &'static str = "\u{00a7}e";
    pub const WHITE: &'static str = "\u{00a7}f";

    pub const OBFUSCATED: &'static str = "\u{00a7}k";
    pub const BOLD: &'static str = "\u{00a7}l";
    pub const STRIKETHROUGH: &'static str = "\u{00a7}m";
    pub const UNDERLINE: &'static str = "\u{00a7}n";
    pub const ITALIC: &'static str = "\u{00a7}o";
    pub const RESET: &'static str = "\u{00a7}r";

    pub fn colorize(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let bytes = input.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'&' || bytes[i] == 0xc2 && i + 1 < bytes.len() && bytes[i + 1] == 0xa7  {
                if bytes[i] == b'&' && i + 1 < bytes.len() && Self::is_color_code(bytes[i + 1]) {
                    out.push(Self::ESCAPE);
                    out.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        out
    }

    fn is_color_code(c: u8) -> bool {
        matches!(c, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' | b'k'..=b'o' | b'K'..=b'O' | b'r' | b'R')
    }

    pub fn clean(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c == Self::ESCAPE {
                chars.next();
            } else {
                out.push(c);
            }
        }
        out
    }

    pub fn to_ansi(input: &str) -> String {
        let mut out = String::new();
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c == Self::ESCAPE {
                if let Some(code) = chars.next() {
                    out.push_str(Self::code_to_ansi(code));
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    fn code_to_ansi(c: char) -> &'static str {
        match c {
            '0' => "\x1b[30m",
            '1' => "\x1b[34m",
            '2' => "\x1b[32m",
            '3' => "\x1b[36m",
            '4' => "\x1b[31m",
            '5' => "\x1b[35m",
            '6' => "\x1b[33m",
            '7' => "\x1b[37m",
            '8' => "\x1b[90m",
            '9' => "\x1b[94m",
            'a' | 'A' => "\x1b[92m",
            'b' | 'B' => "\x1b[96m",
            'c' | 'C' => "\x1b[91m",
            'd' | 'D' => "\x1b[95m",
            'e' | 'E' => "\x1b[93m",
            'f' | 'F' => "\x1b[97m",
            'r' | 'R' => "\x1b[0m",
            _ => "",
        }
    }
}
