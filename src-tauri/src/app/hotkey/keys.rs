//! Key names stored in settings (`Shift`, `Win`, `F13`…) and their Windows virtual-key codes.

// The state machine that uses most of this only runs on Windows.
#![cfg_attr(not(windows), allow(dead_code))]

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods(u8);

impl Mods {
    pub const CTRL: Self = Self(1);
    pub const ALT: Self = Self(2);
    pub const SHIFT: Self = Self(4);
    pub const WIN: Self = Self(8);

    /// Canonical order, also used when displaying a hotkey.
    const NAMED: [(Self, &'static str); 4] = [
        (Self::CTRL, "Ctrl"),
        (Self::ALT, "Alt"),
        (Self::SHIFT, "Shift"),
        (Self::WIN, "Win"),
    ];

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub fn names(self) -> impl Iterator<Item = &'static str> {
        Self::NAMED
            .into_iter()
            .filter(move |(mods, _)| self.contains(*mods))
            .map(|(_, name)| name)
    }

    fn from_name(name: &str) -> Option<Self> {
        Self::NAMED
            .into_iter()
            .find(|(_, known)| known.eq_ignore_ascii_case(name))
            .map(|(mods, _)| mods)
    }
}

pub enum Key {
    Modifier(Mods),
    Other(u16),
}

pub fn classify(vk: u16) -> Key {
    match vk {
        0x10 | 0xA0 | 0xA1 => Key::Modifier(Mods::SHIFT),
        0x11 | 0xA2 | 0xA3 => Key::Modifier(Mods::CTRL),
        0x12 | 0xA4 | 0xA5 => Key::Modifier(Mods::ALT),
        0x5B | 0x5C => Key::Modifier(Mods::WIN),
        _ => Key::Other(vk),
    }
}

const NAMED_KEYS: &[(&str, u16)] = &[
    ("Space", 0x20),
    ("PageUp", 0x21),
    ("PageDown", 0x22),
    ("End", 0x23),
    ("Home", 0x24),
    ("Left", 0x25),
    ("Up", 0x26),
    ("Right", 0x27),
    ("Down", 0x28),
    ("PrintScreen", 0x2C),
    ("Insert", 0x2D),
    ("Delete", 0x2E),
    ("Pause", 0x13),
    ("ScrollLock", 0x91),
];

const DIGITS: std::ops::RangeInclusive<u16> = 0x30..=0x39;
const LETTERS: std::ops::RangeInclusive<u16> = 0x41..=0x5A;
const FUNCTION_KEYS: std::ops::RangeInclusive<u16> = 0x70..=0x87;

/// Name of a key Arthex accepts in a hotkey, or `None` for keys it ignores.
pub fn key_name(vk: u16) -> Option<String> {
    if DIGITS.contains(&vk) || LETTERS.contains(&vk) {
        return Some(char::from(u8::try_from(vk).ok()?).to_string());
    }
    if FUNCTION_KEYS.contains(&vk) {
        return Some(format!("F{}", vk - 0x6F));
    }
    NAMED_KEYS
        .iter()
        .find(|(_, code)| *code == vk)
        .map(|(name, _)| (*name).to_owned())
}

pub fn key_code(name: &str) -> Option<u16> {
    let mut chars = name.chars();
    if let (Some(single), None) = (chars.next(), chars.next()) {
        let code = u16::try_from(u32::from(single.to_ascii_uppercase())).ok()?;
        return (DIGITS.contains(&code) || LETTERS.contains(&code)).then_some(code);
    }
    if let Some(number) = name
        .strip_prefix(['F', 'f'])
        .and_then(|n| n.parse::<u16>().ok())
    {
        return (1..=24).contains(&number).then_some(0x6F + number);
    }
    NAMED_KEYS
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
        .map(|(_, code)| *code)
}

/// Keys that would get in the way of typing if they opened Arthex on their own or with Shift.
fn is_typing_key(vk: u16) -> bool {
    DIGITS.contains(&vk) || LETTERS.contains(&vk) || vk == 0x20
}

/// Either modifiers only (fires when released, like Shift + Win) or modifiers plus one key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hotkey {
    pub mods: Mods,
    pub key: Option<u16>,
}

impl Hotkey {
    pub const DEFAULT: Self = Self {
        mods: Mods::SHIFT.union(Mods::WIN),
        key: None,
    };

    /// Parses names from settings. Rejects unknown or repeated keys and unusable combinations.
    pub fn parse<S: AsRef<str>>(names: &[S]) -> Option<Self> {
        let mut hotkey = Self {
            mods: Mods::default(),
            key: None,
        };
        for name in names {
            let name = name.as_ref();
            if let Some(mods) = Mods::from_name(name) {
                if hotkey.mods.contains(mods) {
                    return None;
                }
                hotkey.mods.insert(mods);
            } else if hotkey.key.is_none() {
                hotkey.key = Some(key_code(name)?);
            } else {
                return None;
            }
        }
        hotkey.is_usable().then_some(hotkey)
    }

    /// A lone modifier would fire all the time, and a lone letter would break typing.
    pub fn is_usable(self) -> bool {
        match self.key {
            None => self.mods.count() >= 2,
            Some(vk) => !(is_typing_key(vk) && (self.mods.is_empty() || self.mods == Mods::SHIFT)),
        }
    }

    pub fn names(self) -> Vec<String> {
        self.mods
            .names()
            .map(str::to_owned)
            .chain(self.key.and_then(key_name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(names: &[&str]) -> Option<Vec<String>> {
        Hotkey::parse(names).map(Hotkey::names)
    }

    #[test]
    fn parses_into_canonical_order() {
        assert_eq!(parse(&["Win", "Shift"]).unwrap(), ["Shift", "Win"]);
        assert_eq!(
            parse(&["space", "alt", "ctrl"]).unwrap(),
            ["Ctrl", "Alt", "Space"]
        );
        assert_eq!(parse(&["F13"]).unwrap(), ["F13"]);
        assert_eq!(parse(&["Ctrl", "k"]).unwrap(), ["Ctrl", "K"]);
    }

    #[test]
    fn rejects_unusable_or_unknown_keys() {
        assert_eq!(parse(&["Shift"]), None);
        assert_eq!(parse(&["A"]), None);
        assert_eq!(parse(&["Shift", "A"]), None);
        assert_eq!(parse(&["Space"]), None);
        assert_eq!(parse(&["Ctrl", "Ctrl", "K"]), None);
        assert_eq!(parse(&["Ctrl", "K", "L"]), None);
        assert_eq!(parse(&["Ctrl", "Nope"]), None);
        assert_eq!(parse(&["F25"]), None);
        assert_eq!(parse(&[]), None);
    }

    #[test]
    fn names_round_trip() {
        for vk in 0..=0xFF {
            if let Some(name) = key_name(vk) {
                assert_eq!(key_code(&name), Some(vk), "{name}");
            }
        }
    }
}
